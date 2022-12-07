use crate::models::JeopardyQuestion;
use crate::parser::JArchiveDocumentParser;
use std::error::Error;
use std::fmt;

#[derive(Default)]
pub struct JArchiveScraper;

#[derive(Debug, Clone)]
pub struct ScrapingError {
    pub episode_no: u32,
}

impl fmt::Display for ScrapingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!("Could not scape data for episode {0}", self.episode_no)
        )
    }
}

impl ScrapingError {
    pub fn new(episode_no: u32) -> ScrapingError {
        ScrapingError { episode_no }
    }
}

impl JArchiveScraper {
    pub fn new() -> Self {
        Default::default()
    }

    /// Scrapes j-archive for jeopardy questions
    pub async fn scrape(
        self,
        episode_no: u32,
        iterations: u32,
    ) -> Result<Vec<JeopardyQuestion>, ScrapingError> {
        let mut results: Vec<JeopardyQuestion> = Vec::new();

        for i in episode_no..(episode_no + iterations) {
            println!("> Scraping jeopardy questions for episode {0}", i);

            let raw_html = JArchiveScraper::get_html(i)
                .await
                .map_err(|_| ScrapingError::new(episode_no))?;

            if raw_html.contains(&format!("ERROR: No game {0} in database.", episode_no)) {
                return Err(ScrapingError::new(episode_no));
            }

            let document = scraper::Html::parse_document(&raw_html);

            let questions = JArchiveDocumentParser::new(document).parse_all_rounds();

            println!(">> Successfully scraped questions episode {0}", i);

            results.extend(questions);
        }

        Ok(results)
    }

    /// Gets the raw html for a page
    pub async fn get_html(episode_no: u32) -> Result<String, Box<dyn Error>> {
        let raw_html = reqwest::get(format!(
            "https://j-archive.com/showgame.php?game_id={0}",
            episode_no
        ))
        .await?
        .text()
        .await?;

        Ok(raw_html)
    }
}
