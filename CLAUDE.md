# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build
cargo run                          # crawls the default episode (7515), prints JSON to stdout
cargo run -- -e 9200               # specific episode
cargo run -- -e 9200 -i 10         # 10 consecutive episodes starting at 9200
cargo run -- -e 9200 -o out.json   # write to file instead of stdout
cargo run -- -e 1 -i 500 -d 2000 -j 500  # throttled bulk crawl
cargo test
cargo test trims_str               # single test by name
cargo clippy
```

`-d/--delay` (default 1000ms) and `-j/--jitter` (default 0..=500ms extra) set the
wait between consecutive fetches. It is skipped before the first episode and
after the last, so single-episode runs are unaffected; `-d 0 -j 0` disables it.

Progress is written to **stderr** and JSON to **stdout**, so `... > out.json` is
safe to pipe.

Tests live inline under `#[cfg(test)]` in `src/utils/mod.rs`, `src/models/delay.rs` and `src/parser/mod.rs`; there is no `tests/` directory. Coverage is the sanitizer (`utils`), the delay sampler (`models::delay`), and the parser's failure paths (`parser`), the last of which build small HTML fixtures rather than hitting the network.

`exec.sh` batch-crawls episodes 1-9538 in chunks of 500, overridable via `START`, `END`,
`CHUNK`, `DELAY`, `JITTER` and `OUT` env vars. The chunk boundaries are derived from those
values, not hand-written, so filenames always match the range actually crawled — they used
to be written out by line and two chunks were silently mislabeled. Keep them derived. A
chunk that fails entirely is collected and reported at the end rather than aborting the run.

## Docs

Keep `README.md` short and factual: how to run it, what comes out, what bites you. No
marketing tone, no filler sections, no editorializing about the upstream site. Every
claim must be traceable to the code, the CLI output, or an actual crawl — do not
characterize j-archive, its maintainers, or anything else the repo does not state.

## Architecture

Single binary, no library target. The pipeline is linear:

`main` → `JArchiveCrawler::crawl` → (per episode) `JArchiveDocumentParser::parse` → `JeopardyEpisode` → `Reporter::write` → `Serializer::to_json`

- **crawler/** — owns the network loop. Fetches `j-archive.com/showgame.php?game_id={n}` through one shared `reqwest::Client` built per crawl, with an explicit User-Agent, connect/request timeouts, up to `MAX_ATTEMPTS` retries and a `CrawlDelay` between episodes. Still no concurrency: iterating hits the live site once per episode, serially. See **Context** below before touching any of it.
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

Three error types coexist and do not compose: `models::error::Error` (thiserror; `Static(&'static str)` plus `Message(String)` for context-carrying failures, built via `Error::message(..)`), `CrawlerError` (hand-rolled, in `crawler/`), and `JeopardyQuestionBuilderError` (in `models/question.rs`).

The crawl loop is failure-tolerant by design: a request error, a missing episode, or a parse error is recorded in a `failures` list and the loop continues, so one bad episode cannot abandon a long run. Skips are summarised on stderr at the end. Preserve this when editing `crawl` — do not reintroduce `?` or an early `return` inside the loop. If *every* episode fails, `crawl` returns `Err` so a bulk run exits non-zero rather than writing an empty array.

The parser does not panic on bad page content. Every failure path returns `Error::Message` naming the episode, the round and the clue index, so a skip in a 9,000-episode run says what actually went wrong rather than just "failed to parse". When adding parsing code, keep this property: index with `.get()` and propagate with `?`, never `[...]` or `.expect(..)` on anything derived from the page.

The only remaining `unwrap()`s in `parser/` are `Selector::parse` on string literals. Those can fail only from a typo in the selector itself, which would break every page immediately and be caught by the first test run, so they are left as-is.

### Sanitizer regexes are greedy

`utils::sanitizer` strips tags with `</.+>` and `<.+>`. These are greedy and unanchored, so a string with two separate tags loses everything between the first `<` and the last `>`. It also uses `replace` (single match) rather than `replace_all`. The existing tests only exercise single-tag inputs.

## Context

`get_html` takes the shared `reqwest::Client` built once per crawl (connection reuse) and retries each episode up to `MAX_ATTEMPTS` with a 2s/4s backoff. **reqwest applies no timeout by default**, so `REQUEST_TIMEOUT` and `CONNECT_TIMEOUT` are set explicitly — without them a connection that opens and then goes silent hangs the whole crawl forever, with no error and no progress. This was observed in practice. Never build the client without them.

**j-archive returns HTTP 403 to any request that omits a `User-Agent` header**, and `reqwest` omits one by default. `get_html` therefore sets an explicit UA; do not replace it with a bare `reqwest::get`. Note also that `get_html` does not check the response status, so a 403 (or any error page) is handed to the parser and surfaces as the misleading message "Failed to scrape j-archive.com for jeopardy episode N" rather than as an HTTP error.

j-archive changed its markup in April 2023 to stop embedding correct responses in the HTML body; that was subsequently fixed here (see README). Because the parser depends on exact selectors and positional ordering, upstream markup changes are the most likely cause of a sudden breakage, and they tend to surface as wrong data rather than as a crash. When debugging, fetch the raw HTML for the episode and diff the selector hit counts (`td.clue_text:first-of-type`, `td.category td.category_name`, `.correct_response`) before assuming the bug is in Rust.
