mod models;
mod parser;
mod reporter;

use clap::Parser;
use models::{CliArgs, JeopardyQuestion, Round};
use parser::parse_questions;
use reporter::Reporter;
use scraper::Selector;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    let episode_no = args.episode_no;
    let iterations = args.iterations;
    let outfile = args.outfile;

    let mut results: Vec<JeopardyQuestion> = Vec::new();

    for i in episode_no..=(episode_no + u32::from(iterations)) {
        println!("> Scraping jeopardy questions for episode {0}", i);

        let scraped_results = scrape(episode_no).await?;

        match scraped_results {
            Some(questions) => {
                println!(">> Successfully scraped questions episode {0}", i);

                results.extend(questions.iter().cloned());
            }
            None => {
                println!(
                    ">> Failed to scrape questions for episode {0}. Skipping.",
                    i
                );

                continue;
            }
        };
    }

    if let Some(out) = outfile {
        Reporter::new(results)
            .write(out)
            .await
            .expect("Unable to write results to outfile");
    } else {
        // if we don't provide an outfile, assume the user wants results printed to the console
        let json = serde_json::to_string_pretty(&results).unwrap();

        println!("{}", json);
    }

    println!("Run complete!");

    Ok(())
}

/// Gets the raw html for a page
async fn get_html(episode_no: u32) -> Result<String, Box<dyn Error>> {
    let url = format!("https://j-archive.com/showgame.php?game_id={0}", episode_no);

    let raw_html = reqwest::get(url).await?.text().await?;

    Ok(raw_html)
}

/// Scrapes j-archive for jeopardy questions
async fn scrape(episode_no: u32) -> Result<Option<Vec<JeopardyQuestion>>, Box<dyn Error>> {
    let raw_html = get_html(episode_no).await?;

    if raw_html.contains(&format!("ERROR: No game {0} in database.", episode_no)) {
        return Ok(None);
    }

    let document = scraper::Html::parse_document(&raw_html);

    let jr_selector = Selector::parse("#jeopardy_round").unwrap();
    let djr_selector = Selector::parse("#double_jeopardy_round").unwrap();
    let fjr_selector = Selector::parse("#final_jeopardy_round").unwrap();

    let jr_table = document.select(&jr_selector).next().unwrap();
    let djr_table = document.select(&djr_selector).next().unwrap();
    let fjr_table = document.select(&fjr_selector).next().unwrap();

    let jr_questions: Vec<JeopardyQuestion> = parse_questions(&jr_table, Round::Jeopardy);
    let djr_questions: Vec<JeopardyQuestion> = parse_questions(&djr_table, Round::DoubleJeopardy);
    let fjr_questions = parse_questions(&fjr_table, Round::FinalJeopardy);

    let mut questions: Vec<JeopardyQuestion> = Vec::new();

    for question in jr_questions {
        questions.push(question);
    }

    for question in djr_questions {
        questions.push(question);
    }

    // note: there should only ever be one
    for question in fjr_questions {
        questions.push(question);
    }

    Ok(Some(questions))
}
