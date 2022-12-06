# j-archive-scraper

This tool crawls [j-archive](https://j-archive.com/) to generate Jeopardy question in JSON format.

# Usage

Fetch questions for the latest Jeopardy episode

```
cargo run
```

Fetch questions for a specific episode

```
cargo run -- --episode 7000
# or cargo run -- -e 7000
```

Fetch questions for multiple episodes

```
cargo run -- --iterations 10
# or cargo run -- -i 10
```

Save results to a file

```
cargo run -- --outfile ./results.json
# or cargo run -- -o ./results.json
```

# Example data

Some initial example data has been written to the `./results/results.json` file that is included in this repository. This data was generated via `$ cargo run -- --outfile ./example/results.json --episode 6500 --iterations 50`.

```
[
  {
    "question": "A \"glow plug\" is used for cold starts in this engine that doesn't use spark plugs",
    "category": "AUTOMOBILES",
    "round": "Jeopardy",
    "value": 200,
    "answer": "a diesel"
  },
  {
    "question": "Forming most of Vietnam's northwestern border, it's the only landlocked country in Southeast Asia",
    "category": "RIVERS",
    "round": "DoubleJeopardy",
    "value": 2000,
    "answer": "Laos"
  },
  {
    "question": "Despite govt. predictions to the contrary, in 1985 this became the least populated U.S. state",
    "category": "U.S. STATES",
    "round": "FinalJeopardy",
    "value": null,
    "answer": "Wyoming"
  }
]
```

# cli --help

```
Program to scrape jeopardy question data from j-archive.com

Usage: j-archive-scraper [OPTIONS]

Options:
  -e, --episode <EPISODE_NO>     The episode number to parse (note: if iteratons are applied, this will be the starting episode) [default: 7515]
  -i, --iterations <ITERATIONS>  The number of iterations [default: 1]
  -o, --outfile <OUTFILE>        Where to write the results to
  -h, --help                     Print help information
  -V, --version                  Print version information ./README.md
```
