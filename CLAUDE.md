# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build
cargo run                          # crawls the default episode (7515), prints JSON to stdout
cargo run -- -e 9200               # specific episode
cargo run -- -e 9200 -i 10         # 10 consecutive episodes starting at 9200
cargo run -- -e 9200 -o out.json   # write to file instead of stdout
cargo test
cargo test trims_str               # single test by name
cargo clippy
```

Tests live inline in `src/utils/mod.rs` under `#[cfg(test)]`; there is no `tests/` directory. The only covered code is the sanitizer.

`exec.sh` batch-crawls episodes 1-9000 in chunks of 500. Note it has two typos in its episode ranges (`-e 6601` labeled `5501_6000`, and `-e 75001` labeled `7501_8000`), so it does not actually cover what its filenames claim.

## Architecture

Single binary, no library target. The pipeline is linear:

`main` → `JArchiveCrawler::crawl` → (per episode) `JArchiveDocumentParser::parse` → `JeopardyEpisode` → `Reporter::write` → `Serializer::to_json`

- **crawler/** — owns the network loop. Fetches `j-archive.com/showgame.php?game_id={n}` with a bare `reqwest::get`, no user agent, rate limiting, retry, or concurrency. Iterating hits the live site once per episode, serially.
- **parser/** — all the real logic. Turns one page's `scraper::Html` into an episode.
- **models/** — data types, each with a hand-written builder (`set_*` returning `&mut Self`, then `build() -> Result<_, _>`).
- **reporter/** — dual purpose: `report_progress` draws the crossterm spinner during the crawl, `write` emits final JSON. The crawler builds its own `Reporter` for progress while `main` builds a second one for output.
- **serializer/** — thin `serde_json::to_string_pretty` wrapper.
- **utils/sanitizer** — strips HTML tags and entities out of scraped strings.

### Parsing is positional, not structural

This is the thing to understand before touching `parser/mod.rs`. The parser does **not** walk the clue table cell by cell. For each round it flattens the whole table into independent parallel vectors and then re-associates them by index:

- `parse_prompts` collects every `td.clue_text:first-of-type` into a flat `Vec<String>`.
- `parse_categories` collects every `td.category td.category_name` into a flat `Vec<String>`.
- `parse_answer` selects the *nth* `.correct_response` in the table.
- Category for clue `i` is `categories[i.rem_euclid(6)]` (or `categories[0]` when there is only one, i.e. Final Jeopardy).
- Dollar value is **computed from the index**, never read from the page: indices 0-5 → 200, 6-11 → 400, and so on, doubled for Double Jeopardy, `None` for Final.

The consequence: anything that shifts the index — an unrevealed clue on the board, a layout change on j-archive, a selector that silently matches a different count — misassigns category *and* value for every clue after it, with no error. Scraped output can be confidently wrong rather than empty. Prefer fixing this by reading values/categories from the DOM near each clue over adding more index arithmetic.

`calculate_question_value` also returns `Some(0)` rather than `None` for indices past 29.

### Rounds are a fixed 3-tuple

`JeopardyEpisode.rounds` is `(JeopardyRound, JeopardyRound, JeopardyRound)` — Jeopardy, Double, Final, in that order. `parse_rounds` bails with an error if any one of the three is missing, so episodes without a standard three-round structure (tiebreakers, incomplete archive pages) cannot be represented and will fail the whole episode.

### Error handling

Three unrelated error types coexist and do not compose: `models::error::Error` (thiserror, with a single `Static(&'static str)` variant), `CrawlerError` (hand-rolled, in `crawler/`), and `JeopardyQuestionBuilderError` (in `models/question.rs`). Errors are generally flattened into `Error::Static("...")` via `let Ok(x) = ... else`, which discards the underlying cause.

Builders are called with `.expect(...)` throughout the parser, so a malformed page panics the process rather than skipping the episode. The one exception is in `crawler`, where a failed `parse()` prints a message and continues.

A known bug in `crawler/mod.rs`: the "No game in database" guard interpolates `episode_no` (the starting episode) instead of `episode` (the current one), so across a multi-episode run it only detects a missing episode on the first iteration. It also `return`s an `Err`, aborting the remaining episodes instead of skipping the missing one.

### Sanitizer regexes are greedy

`utils::sanitizer` strips tags with `</.+>` and `<.+>`. These are greedy and unanchored, so a string with two separate tags loses everything between the first `<` and the last `>`. It also uses `replace` (single match) rather than `replace_all`. The existing tests only exercise single-tag inputs.

## Context

**j-archive returns HTTP 403 to any request that omits a `User-Agent` header**, and `reqwest` omits one by default. `get_html` therefore sets an explicit UA; do not replace it with a bare `reqwest::get`. Note also that `get_html` does not check the response status, so a 403 (or any error page) is handed to the parser and surfaces as the misleading message "Failed to scrape j-archive.com for jeopardy episode N" rather than as an HTTP error.

j-archive changed its markup in April 2023 to stop embedding correct responses in the HTML body; that was subsequently fixed here (see README). Because the parser depends on exact selectors and positional ordering, upstream markup changes are the most likely cause of a sudden breakage, and they tend to surface as wrong data rather than as a crash. When debugging, fetch the raw HTML for the episode and diff the selector hit counts (`td.clue_text:first-of-type`, `td.category td.category_name`, `.correct_response`) before assuming the bug is in Rust.
