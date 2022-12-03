use regex::Regex;
use scraper::{ElementRef, Selector};

use crate::models::{JeopardyQuestion, Round};

const NUM_CATEGORIES: usize = 6;

/// Parses raw jarchive HTML data into structured objects
pub fn parse_questions(table: &ElementRef, round: Round) -> Vec<JeopardyQuestion> {
    let category_selector = Selector::parse("table.round td.category td.category_name").unwrap();
    let question_selector = Selector::parse("td.clue_text").unwrap();
    let answer_selector = Selector::parse("td.clue div").unwrap();

    let categories: Vec<String> = table
        .select(&category_selector)
        .map(|c| c.inner_html())
        .collect();

    let questions: Vec<String> = table
        .select(&question_selector)
        .map(|c| c.inner_html())
        .collect();

    let mut jeopardy_questions: Vec<JeopardyQuestion> = Vec::new();

    for i in 0..questions.len() {
        let question = &questions[i];
        let category_idx = i.rem_euclid(NUM_CATEGORIES);
        let category = &categories[category_idx];

        let answer = parse_answer(table.select(&answer_selector).nth(i).unwrap());

        let mut value = match i {
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

        let jeopardy_question = JeopardyQuestion {
            category: category.to_owned(),
            question: question.to_owned(),
            round,
            value,
            answer,
        };

        jeopardy_questions.push(jeopardy_question);
    }

    return jeopardy_questions;
}

/// Parses an answer string from an element ref
/// Note: For some reason unknown to me the regex crate does not support lookaheads...
/// Just match this for now and we can strip off the values using string magic
fn parse_answer(clue: ElementRef) -> Option<String> {
    // TODO: use this someday let correct_answer_regex = Regex::new(r#"(?<=<em class=\\"correct_response\\">).+(?=<\/em>)"#).unwrap();
    let correct_answer_regex = Regex::new(r#"<em class="correct_response">.+</em>"#).unwrap();

    match clue.value().attr("onmouseover") {
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
