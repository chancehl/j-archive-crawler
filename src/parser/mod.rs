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
        let Ok(rounds) = self.parse_rounds() else {
            return Err(Error::Static("Failed to parse episode data"));
        };

        Ok(JeopardyEpisodeBuilder::new()
            .set_id(self.episode_no)
            .set_rounds(rounds)
            .set_air_date(self.parse_air_date())
            .build()
            .expect("Could not build jeopardy episode from the given data"))
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
        let mut round_builder = JeopardyRoundBuilder::new();

        let Ok(jeopardy_questions) = self.parse_questions(Round::Jeopardy) else {
            return Err(Error::Static("Could not parse jeopardy questions"));
        };

        let Ok(double_jeopardy_questions) = self.parse_questions(Round::DoubleJeopardy) else {
            return Err(Error::Static("Could not parse double jeopardy questions"));
        };

        let Ok(final_jeopardy_question) = self.parse_questions(Round::FinalJeopardy) else {
            return Err(Error::Static("Could not parse final jeopardy question"));
        };

        let jeopardy_round = round_builder
            .set_questions(jeopardy_questions)
            .set_round(Round::Jeopardy)
            .build()
            .expect("Could not build jeopardy round from the provided data");

        let double_jeopardy_round = round_builder
            .set_questions(double_jeopardy_questions)
            .set_round(Round::DoubleJeopardy)
            .build()
            .expect("Could not build double jeopardy round from the provided data");

        let final_jeopardy_round = round_builder
            .set_questions(final_jeopardy_question)
            .set_round(Round::FinalJeopardy)
            .build()
            .expect("Could not build final jeopardy round from the provided data");

        Ok((jeopardy_round, double_jeopardy_round, final_jeopardy_round))
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
            return Err(Error::Static("Could not locate jeopardy table"));
        };

        let categories = self.parse_categories(table);
        let prompts = self.parse_prompts(table);

        let mut jeopardy_questions: Vec<JeopardyQuestion> = Vec::new();

        for i in 0..prompts.len() {
            let prompt = &prompts[i];
            let category = &categories[if categories.len() == 1 {
                0
            } else {
                i.rem_euclid(NUM_CATEGORIES)
            }];
            let answer = self.parse_answer(table, i, round);
            let value = self.calculate_question_value(i, round);

            let question = JeopardyQuestionBuilder::new()
                .set_answer(answer)
                .set_category(category)
                .set_prompt(prompt)
                .set_round(round)
                .set_value(value)
                .build()
                .expect("Could not build jeopardy question model");

            let question = question.sanitize();

            jeopardy_questions.push(question);
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
