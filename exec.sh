#!/usr/bin/env bash
#
# Crawls j-archive in chunks. The episode numbers used to be written out by
# hand, one line per chunk, which is how "-e 6601" ended up labelled
# 5501_6000 and "-e 75001" ended up labelled 7501_8000 -- two whole chunks
# silently lost. The range is derived now so that cannot happen again.
#
# Override any of these:
#   START=9251 END=9538 CHUNK=500 DELAY=1000 JITTER=500 OUT=./out ./exec.sh
set -uo pipefail

START="${START:-1}"
END="${END:-9538}"
CHUNK="${CHUNK:-500}"
DELAY="${DELAY:-1000}"
JITTER="${JITTER:-500}"
OUT="${OUT:-.}"

mkdir -p "$OUT"

cargo build --release
BIN="./target/release/j-archive-crawler"

failed=()

for (( lo = START; lo <= END; lo += CHUNK )); do
    hi=$(( lo + CHUNK - 1 ))
    (( hi > END )) && hi=$END

    count=$(( hi - lo + 1 ))
    name="results-${lo}_${hi}.json"

    echo ">>> ${name} (episodes ${lo}..${hi})"

    # A chunk only exits non-zero when every episode in it failed; keep going
    # so one bad chunk does not abandon the rest of the crawl.
    if ! "$BIN" -e "$lo" -i "$count" -d "$DELAY" -j "$JITTER" -o "${OUT}/${name}"; then
        echo "!!! chunk ${lo}..${hi} failed entirely" >&2
        failed+=( "${lo}-${hi}" )
    fi
done

if (( ${#failed[@]} > 0 )); then
    echo >&2
    echo "${#failed[@]} chunk(s) failed: ${failed[*]}" >&2
    exit 1
fi

echo "All chunks written to ${OUT}"
