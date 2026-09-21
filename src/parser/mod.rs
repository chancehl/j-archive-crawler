use scraper::{ElementRef, Html, Selector};

use crate::models::{
    episode::{JeopardyEpisode, JeopardyEpisodeBuilder},
    error::Error,
    question::{JeopardyQuestion, JeopardyQuestionBuilder, Round},
    round::{JeopardyRound, JeopardyRoundBuilder},
};

const NUM_CATEGORIES: usize = 6;

pub struct JArchiveDocumentParser {
    document: Html,
    episode_no: u32,
}

impl JArchiveDocumentParser {
    /// Creates a new parser object
    pub fn new(document: Html, episode_no: u32) -> Self {
        JArchiveDocumentParser {
            document,
            episode_no,
        }
    }

    /// Parses the provided document into jeopardy episode data
    pub fn parse(&self) -> Result<JeopardyEpisode, Error> {
        let rounds = self.parse_rounds()?;

        JeopardyEpisodeBuilder::new()
            .set_id(self.episode_no)
            .set_rounds(rounds)
            .set_air_date(self.parse_air_date())
            .build()
    }

    /// Parses the air date
    fn parse_air_date(&self) -> Option<String> {
        let air_date_selector = Selector::parse("#game_title h1").unwrap();

        let Some(air_date_element) = self.document.select(&air_date_selector).next() else {
            return None;
        };

        let date = air_date_element.inner_html();

        let Some(date) = date.split(" - ").nth(1) else {
            return None;
        };

        Some(date.to_string())
    }

    /// Parses all rounds
    fn parse_rounds(&self) -> Result<(JeopardyRound, JeopardyRound, JeopardyRound), Error> {
        let jeopardy_round = self.parse_round(Round::Jeopardy)?;
        let double_jeopardy_round = self.parse_round(Round::DoubleJeopardy)?;
        let final_jeopardy_round = self.parse_round(Round::FinalJeopardy)?;

        Ok((jeopardy_round, double_jeopardy_round, final_jeopardy_round))
    }

    /// Parses a single round
    fn parse_round(&self, round: Round) -> Result<JeopardyRound, Error> {
        let questions = self.parse_questions(round)?;

        JeopardyRoundBuilder::new()
            .set_questions(questions)
            .set_round(round)
            .build()
    }

    /// Parses categories
    fn parse_categories(&self, fragment: ElementRef) -> Vec<String> {
        let category_selector = Selector::parse("td.category td.category_name").unwrap();

        fragment
            .select(&category_selector)
            .map(|c| c.inner_html())
            .collect()
    }

    /// Parses table fragment
    fn parse_table(&self, round: Round) -> Option<ElementRef> {
        let table_selector = match round {
            Round::Jeopardy => Selector::parse("#jeopardy_round").unwrap(),
            Round::DoubleJeopardy => Selector::parse("#double_jeopardy_round").unwrap(),
            Round::FinalJeopardy => Selector::parse("#final_jeopardy_round").unwrap(),
        };

        self.document.select(&table_selector).next()
    }

    /// Calculate question value
    fn calculate_question_value(&self, index: usize, round: Round) -> Option<u32> {
        let mut value = match index {
            0..=5 => 200,
            6..=11 => 400,
            12..=17 => 600,
            18..=23 => 800,
            24..=29 => 1000,
            _ => 0,
        };

        if round == Round::DoubleJeopardy {
            value = value * 2;
        }

        if round == Round::FinalJeopardy {
            None
        } else {
            Some(value)
        }
    }

    /// Parses prompts
    fn parse_prompts(&self, fragment: ElementRef) -> Vec<String> {
        let question_selector = Selector::parse("td.clue_text:first-of-type").unwrap();

        fragment
            .select(&question_selector)
            .map(|c| c.inner_html())
            .collect()
    }

    /// Parses raw jarchive HTML data into structured objects
    fn parse_questions(&self, round: Round) -> Result<Vec<JeopardyQuestion>, Error> {
        let Some(table) = self.parse_table(round) else {
            return Err(Error::message(format!(
                "episode {0}: no {1:?} table on the page",
                self.episode_no, round
            )));
        };

        let categories = self.parse_categories(table);
        let prompts = self.parse_prompts(table);

        if prompts.is_empty() {
            return Err(Error::message(format!(
                "episode {0}: {1:?} table contains no clues",
                self.episode_no, round
            )));
        }

        if categories.is_empty() {
            return Err(Error::message(format!(
                "episode {0}: {1:?} table contains no categories",
                self.episode_no, round
            )));
        }

        let mut jeopardy_questions: Vec<JeopardyQuestion> = Vec::with_capacity(prompts.len());

        for (index, prompt) in prompts.iter().enumerate() {
            // A round normally lays out six categories across; a short or
            // reordered board must not index past the end of the vector.
            let category_index = if categories.len() == 1 {
                0
            } else {
                index.rem_euclid(NUM_CATEGORIES)
            };

            let Some(category) = categories.get(category_index) else {
                return Err(Error::message(format!(
                    "episode {0}: {1:?} clue {2} maps to category {3}, but only {4} were found",
                    self.episode_no,
                    round,
                    index,
                    category_index,
                    categories.len()
                )));
            };

            let answer = self.parse_answer(table, index, round);
            let value = self.calculate_question_value(index, round);

            let question = JeopardyQuestionBuilder::new()
                .set_answer(answer)
                .set_category(category)
                .set_prompt(prompt)
                .set_round(round)
                .set_value(value)
                .build()
                .map_err(|err| {
                    Error::message(format!(
                        "episode {0}: {1:?} clue {2}: {3}",
                        self.episode_no, round, index, err
                    ))
                })?;

            jeopardy_questions.push(question.sanitize());
        }

        Ok(jeopardy_questions)
    }

    /// Parses an answer string from an element ref
    /// Note: For some reason unknown to me the regex crate does not support lookaheads...
    /// Just match this for now and we can strip off the values using string magic
    fn parse_answer(&self, fragment: ElementRef, index: usize, _round: Round) -> Option<String> {
        let correct_response_selector =
            Selector::parse(".correct_response").expect("Failed to parse selector");

        fragment
            .select(&correct_response_selector)
            .nth(index)
            .map(|element| element.text().collect::<Vec<_>>().join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::JArchiveDocumentParser;
    use crate::models::question::Round;
    use scraper::Html;

    /// Builds a round container with the given categories and clues
    fn round_html(id: &str, categories: &[&str], clues: &[&str]) -> String {
        let categories = categories
            .iter()
            .map(|category| {
                format!(
                    "<td class=\"category\"><table><tr><td class=\"category_name\">{0}</td></tr></table></td>",
                    category
                )
            })
            .collect::<String>();

        let clues = clues
            .iter()
            .map(|clue| format!("<tr><td class=\"clue_text\">{0}</td></tr>", clue))
            .collect::<String>();

        format!(
            "<div id=\"{0}\"><table><tr>{1}</tr>{2}</table></div>",
            id, categories, clues
        )
    }

    fn parser_for(body: &str) -> JArchiveDocumentParser {
        JArchiveDocumentParser::new(Html::parse_document(body), 1234)
    }

    #[test]
    fn empty_document_is_an_error() {
        assert!(parser_for("<html></html>").parse().is_err());
    }

    #[test]
    fn missing_round_reports_which_round() {
        let err = parser_for("<html></html>")
            .parse_questions(Round::FinalJeopardy)
            .unwrap_err()
            .to_string();

        assert!(err.contains("FinalJeopardy"), "unhelpful error: {0}", err);
        assert!(err.contains("1234"), "error omits the episode: {0}", err);
    }

    #[test]
    fn round_without_categories_is_an_error_not_a_panic() {
        let html = round_html("jeopardy_round", &[], &["a clue"]);

        let err = parser_for(&html)
            .parse_questions(Round::Jeopardy)
            .unwrap_err()
            .to_string();

        assert!(err.contains("no categories"), "unexpected error: {0}", err);
    }

    #[test]
    fn round_without_clues_is_an_error_not_a_panic() {
        let html = round_html("jeopardy_round", &["CATEGORY"], &[]);

        let err = parser_for(&html)
            .parse_questions(Round::Jeopardy)
            .unwrap_err()
            .to_string();

        assert!(err.contains("no clues"), "unexpected error: {0}", err);
    }

    /// Previously panicked: ten clues index past a two-category board
    #[test]
    fn fewer_categories_than_clues_is_an_error_not_a_panic() {
        let clues = ["c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9"];
        let html = round_html("jeopardy_round", &["FIRST", "SECOND"], &clues);

        let err = parser_for(&html)
            .parse_questions(Round::Jeopardy)
            .unwrap_err()
            .to_string();

        assert!(
            err.contains("maps to category"),
            "unexpected error: {0}",
            err
        );
    }

    #[test]
    fn well_formed_round_maps_clues_to_categories_in_column_order() {
        let categories = ["C0", "C1", "C2", "C3", "C4", "C5"];
        let clues = ["q0", "q1", "q2", "q3", "q4", "q5", "q6"];
        let html = round_html("jeopardy_round", &categories, &clues);

        let questions = parser_for(&html).parse_questions(Round::Jeopardy).unwrap();

        assert_eq!(questions.len(), 7);
        assert_eq!(questions[0].category, "C0");
        assert_eq!(questions[5].category, "C5");
        // wraps to the next row, back to the first column
        assert_eq!(questions[6].category, "C0");
        assert_eq!(questions[0].value, Some(200));
        assert_eq!(questions[6].value, Some(400));
    }

    #[test]
    fn single_category_round_uses_it_for_every_clue() {
        let html = round_html("final_jeopardy_round", &["ONLY"], &["the clue"]);

        let questions = parser_for(&html)
            .parse_questions(Round::FinalJeopardy)
            .unwrap();

        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].category, "ONLY");
        assert_eq!(questions[0].value, None);
    }
}
