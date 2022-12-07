use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use crate::models::{JeopardyQuestion, JeopardyQuestionBuilder, Round};

const NUM_CATEGORIES: usize = 6;

pub struct JArchiveDocumentParser {
    document: Html,
}

impl JArchiveDocumentParser {
    // Creates a new parser object
    pub fn new(document: Html) -> Self {
        JArchiveDocumentParser { document }
    }

    /// Parses all rounds
    pub fn parse_all_rounds(&self) -> Vec<JeopardyQuestion> {
        let mut questions: Vec<JeopardyQuestion> = Vec::new();

        let jr_questions = &self.parse_questions(Round::Jeopardy);
        let djr_questions = &self.parse_questions(Round::DoubleJeopardy);
        let fjr_questions = &self.parse_questions(Round::FinalJeopardy);

        questions.extend(jr_questions.iter().cloned());
        questions.extend(djr_questions.iter().cloned());
        questions.extend(fjr_questions.iter().cloned());

        questions
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
            let category_idx = i.rem_euclid(NUM_CATEGORIES);
            let category = &categories[category_idx];

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
                    .unwrap()
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
