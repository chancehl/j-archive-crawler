# j-archive-crawler

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

Some initial example data has been written to the `./results/results.json` file that is included in this repository. This data was generated via `$ cargo run -- --outfile ./example/results.json`.

```
[
  {
    "air_date": "Thursday, November 17, 2022",
    "rounds": [
      {
        "questions": [
          {
            "prompt": "Tradition says the pilgrims set foot on this historic artifact on December 26, 1620",
            "category": "HISTORIC DATES",
            "round": "Jeopardy",
            "value": 200,
            "answer": "Plymouth Rock"
          }
        ],
        "round": "Jeopardy"
      },
      {
        "questions": [
          {
            "prompt": "It's a 2-seated pleasure carriage, perhaps \"with the fringe on top\"",
            "category": "DOUBLE LETTERS IN THE MIDDLE",
            "round": "DoubleJeopardy",
            "value": 2000,
            "answer": "a surrey"
          },
        ],
        "round": "DoubleJeopardy"
      },
      {
        "questions": [
          {
            "prompt": "Ridley Scott's first feature film, \"The Duellists\", was based on a story by this author to whom Scott's film \"Alien\" also pays tribute",
            "category": "MOVIES &amp; LITERATURE",
            "round": "FinalJeopardy",
            "value": null,
            "answer": "Joseph Conrad"
          }
        ],
        "round": "FinalJeopardy"
      }
    ],
    "id": 7515
  }
]
```

# cli --help

```
Program to crawl j-archive.com and parse jeopardy question data into json

Usage: j-archive-crawler [OPTIONS]

Options:
  -e, --episode <EPISODE_NO>     The episode number to parse (note: if iteratons are applied, this will be the starting episode) [default: 7515]
  -i, --iterations <ITERATIONS>  The number of iterations [default: 1]
  -o, --outfile <OUTFILE>        Where to write the results to
  -h, --help                     Print help information
  -V, --version                  Print version information ./README.md
```
