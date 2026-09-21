# j-archive-crawler

Scrapes [J! Archive](https://j-archive.com/) and dumps Jeopardy! episodes as JSON —
every clue, its category, its value, and the correct response.

> In April 2023 J! Archive stopped embedding correct responses in the HTML body. That
> broke this tool; it's since been fixed. If you just want the data, ~7,000 episodes are
> pre-exported at [chancehl/JeopardyQuestions](https://github.com/chancehl/JeopardyQuestions).

## Run it

```bash
cargo run -- -e 7515              # one episode, to stdout
cargo run -- -e 9200 -i 10        # episodes 9200..9209
cargo run -- -e 9200 -o out.json  # to a file
cargo run -- -e 1 -i 500 -d 2000  # slower, for bulk
```

| Flag | | Default |
|---|---|---|
| `-e, --episode` | starting episode | 7515 |
| `-i, --iterations` | how many consecutive episodes | 1 |
| `-o, --outfile` | write here instead of stdout | — |
| `-d, --delay` | base ms between fetches | 1000 |
| `-j, --jitter` | extra random ms, `0..=j` | 500 |

Episode numbers are J! Archive `game_id`s (the `N` in `showgame.php?game_id=N`). They
aren't chronological — 7515 aired in 2022, 7517 in 1990.

JSON goes to stdout, progress to stderr, so `> out.json` is safe.

`exec.sh` crawls a big range in chunks, one file each. Defaults to 1..9538 by 500;
override with `START`, `END`, `CHUNK`, `DELAY`, `JITTER`, `OUT`.

## Resuming

A full crawl takes about five hours, so with `-o` each finished episode is appended to
`<outfile>.partial` as it lands. If the run dies, rerun the same command — it reads that
file, skips what it already has, and fetches only the rest:

```
$ cargo run -- -e 1 -i 9538 -o out.json
^C
$ cargo run -- -e 1 -i 9538 -o out.json
Resuming: 4021 of 9538 episodes already crawled
```

The log is deleted once `out.json` is written, so a run that finished will crawl again
from scratch if you rerun it. Crawls to stdout can't be resumed.

## Output

`{ id, air_date, rounds }`, where `rounds` is always exactly three — Jeopardy, Double
Jeopardy, Final Jeopardy, in that order. Final Jeopardy holds one question, value `null`.

```json
[
  {
    "air_date": "Thursday, November 17, 2022",
    "rounds": [
      {
        "round": "Jeopardy",
        "questions": [
          {
            "prompt": "Tradition says the pilgrims set foot on this historic artifact on December 26, 1620",
            "category": "HISTORIC DATES",
            "round": "Jeopardy",
            "value": 200,
            "answer": "Plymouth Rock"
          }
        ]
      }
    ],
    "id": 7515
  }
]
```

Four full episodes in [`example/results.json`](./example/results.json), from
`cargo run -- -e 7518 -i 4 -o ./example/results.json`.

## Gotchas

**Categories and values come from the clue's index, not the page.** Any gap — an
unrevealed clue, a markup change upstream — shifts every clue after it onto the wrong
category and value, with no error. Episode 7517 shows this: 26 revealed Double Jeopardy
clues instead of 30, so from the 7th on, every category is off by one.

**Three rounds or nothing.** Tiebreakers and partial pages fail the whole episode.

**Sanitizing is rough.** Some markup survives, and media clues lose the media — you get
`"This island seen  here  is about 30 miles south of Cape Cod"`.

So bad output usually looks like *wrong* data, not a crash. If it breaks suddenly,
diff the selector hit counts against a raw page before blaming the Rust.

## Crawling behavior

One request at a time, never concurrent. Waits `--delay` plus a random `0..=--jitter`
between episodes; skipped before the first fetch and after the last, so single-episode
runs aren't slowed. `-d 0 -j 0` turns it off. The delay dominates the runtime — parsing
an episode takes ~1.2ms against ~1.9s of waiting and fetching.

j-archive 403s any request without a `User-Agent`, so one is always sent. Requests retry
3× with 2s/4s backoff under a 10s connect / 30s request timeout.

A failed episode is recorded and the crawl continues; skips are summarized on stderr at
the end. If *every* episode fails, the process exits non-zero instead of writing `[]`.

## Development

```bash
cargo test
cargo test trims_str
cargo clippy
```

Tests are inline under `#[cfg(test)]` — no `tests/` directory. They cover the sanitizer
(`utils`), the delay sampler (`models::delay`), and the parser's failure paths against
small HTML fixtures.

```
src/
  crawler/     fetch, retry, throttle, collect failures
  parser/      HTML -> JeopardyEpisode; all the real logic
  models/      data types + builders, CLI args, errors
  reporter/    progress spinner (stderr), final JSON write
  resume/      append-only crash log for restartable crawls
  serializer/  serde_json wrapper
  utils/       tag/entity sanitizer
```

`main` → `crawl` → per episode `parse` → `JeopardyEpisode` → `record` → `write`.

## License

[MIT](./LICENSE) © Chance Linz
