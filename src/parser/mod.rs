use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use crate::models::{
    episode::{JeopardyEpisode, JeopardyEpisodeBuilder},
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
    pub fn parse(&self) -> JeopardyEpisode {
        JeopardyEpisodeBuilder::new()
            .set_id(self.episode_no)
            .set_rounds(self.parse_rounds())
            .set_air_date(self.parse_air_date())
            .build()
            .expect("Could not build jeopardy episode from the given data")
    }

    /// Parses the air date
    fn parse_air_date(&self) -> String {
        let air_date_selector = Selector::parse("#game_title h1").unwrap();

        // TODO: refactor so this is less fragile
        self.document
            .select(&air_date_selector)
            .next()
            .expect("Could not locate air date on document")
            .inner_html()
            .split(" - ") // ex: Show #1234 - Wednesday, December 7th, 2022
            .nth(1) // take what comes after the dash
            .expect("Could not parse air date from episode title")
            .to_owned()
    }

    /// Parses all rounds
    fn parse_rounds(&self) -> (JeopardyRound, JeopardyRound, JeopardyRound) {
        let mut round_builder = JeopardyRoundBuilder::new();

        let jeopardy_round = round_builder
            .set_questions(self.parse_questions(Round::Jeopardy).to_vec())
            .set_round(Round::Jeopardy)
            .build()
            .expect("Could not build jeopardy round from the provided data");

        let double_jeopardy_round = round_builder
            .set_questions(self.parse_questions(Round::DoubleJeopardy))
            .set_round(Round::DoubleJeopardy)
            .build()
            .expect("Could not build double jeopardy round from the provided data");

        let final_jeopardy_round = round_builder
            .set_questions(self.parse_questions(Round::FinalJeopardy))
            .set_round(Round::FinalJeopardy)
            .build()
            .expect("Could not build final jeopardy round from the provided data");

        (jeopardy_round, double_jeopardy_round, final_jeopardy_round)
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
        let question_selector = Selector::parse("td.clue_text").unwrap();

        fragment
            .select(&question_selector)
            .map(|c| c.inner_html())
            .collect()
    }

    /// Parses raw jarchive HTML data into structured objects
    fn parse_questions(&self, round: Round) -> Vec<JeopardyQuestion> {
        let table = self
            .parse_table(round)
            .expect("Could not locate jeopardy table");

        let categories = self.parse_categories(table);
        let prompts = self.parse_prompts(table);

        let mut jeopardy_questions: Vec<JeopardyQuestion> = Vec::new();

        for i in 0..prompts.len() {
            let prompt = &prompts[i];
            let category = &categories[i.rem_euclid(NUM_CATEGORIES)];
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

            jeopardy_questions.push(question);
        }

        return jeopardy_questions;
    }

    /// Parses an answer string from an element ref
    /// Note: For some reason unknown to me the regex crate does not support lookaheads...
    /// Just match this for now and we can strip off the values using string magic
    fn parse_answer(&self, fragment: ElementRef, index: usize, round: Round) -> Option<String> {
        let answer_selector = Selector::parse(if round != Round::FinalJeopardy {
            "td.clue div"
        } else {
            "td.category div"
        })
        .unwrap();

        let node = fragment.select(&answer_selector).nth(index).unwrap();

        // TODO: use this someday let correct_answer_regex = Regex::new(r#"(?<=<em class=\\"correct_response\\">).+(?=<\/em>)"#).unwrap();
        let correct_answer_regex = Regex::new(r#"<em class="correct_response">.+</em>"#).unwrap();

        match node.value().attr("onmouseover") {
            Some(attribute) => {
                // this seems very fragile
                // TODO: refactor
                let match_range = correct_answer_regex
                    .captures_iter(attribute)
                    .next()
                    .expect("Could not find correct response regex match")
                    .get(0)
                    .unwrap()
                    .range();

                let answer = &attribute[match_range];
                let answer = &answer.replace("<em class=\"correct_response\">", "");
                let answer = &answer.replace("</em>", "");

                Some(answer.to_string())
            }
            None => None,
        }
    }
}
