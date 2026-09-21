//! Crash log that lets an interrupted crawl pick up where it left off.
//!
//! A full-archive crawl takes hours, and until now every episode lived in memory
//! until the very end -- a blip at episode 9000 of 9538 threw away the whole run.
//! While crawling we append each finished episode to `<outfile>.partial`, one JSON
//! object per line, flushed as it is written. Re-running the same command reads that
//! file back, skips the episodes it already has, and only fetches the rest.
//!
//! One object per line rather than a JSON array is what makes this cheap: appending
//! never has to rewrite what came before, and a process killed mid-write leaves at
//! worst one unparseable trailing line, which recovery drops.
//!
//! The log is deleted only after the real outfile has been written, so there is no
//! window where neither file holds the data. Crawls that print to stdout cannot be
//! resumed and use `ResumeLog::disabled()`.

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::models::episode::JeopardyEpisode;

/// Where the crash log for a given outfile lives
fn partial_path(outfile: &Path) -> PathBuf {
    let mut name = outfile.as_os_str().to_owned();
    name.push(".partial");

    PathBuf::from(name)
}

pub struct ResumeLog {
    /// `None` when resuming is off, i.e. output is going to stdout
    file: Option<File>,
    path: Option<PathBuf>,
    done: HashSet<u32>,
    recovered: Vec<JeopardyEpisode>,
}

impl ResumeLog {
    /// A log that records nothing, for crawls with no outfile to resume into
    pub fn disabled() -> Self {
        ResumeLog {
            file: None,
            path: None,
            done: HashSet::new(),
            recovered: Vec::new(),
        }
    }

    /// Opens the crash log beside `outfile`, recovering any episodes inside `range`
    /// that a previous run already finished.
    pub fn open(outfile: &str, range: Range<u32>) -> io::Result<Self> {
        let path = partial_path(Path::new(outfile));

        let mut done = HashSet::new();
        let mut recovered = Vec::new();

        // A missing log just means nothing to resume
        if let Ok(contents) = fs::read_to_string(&path) {
            for line in contents.lines() {
                // A line that does not parse is a write torn off by whatever killed
                // the last run. Dropping it is the point of the line-per-episode
                // format, so it is not worth reporting.
                let Ok(episode) = serde_json::from_str::<JeopardyEpisode>(line) else {
                    continue;
                };

                // A log left over from a crawl of some other range must not leak
                // episodes into this one's output
                if !range.contains(&episode.id) || !done.insert(episode.id) {
                    continue;
                }

                recovered.push(episode);
            }
        }

        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(ResumeLog {
            file: Some(file),
            path: Some(path),
            done,
            recovered,
        })
    }

    /// Episodes a previous run already finished, handed over to seed the results
    pub fn take_recovered(&mut self) -> Vec<JeopardyEpisode> {
        std::mem::take(&mut self.recovered)
    }

    /// True when a previous run already crawled this episode
    pub fn is_done(&self, episode_no: u32) -> bool {
        self.done.contains(&episode_no)
    }

    /// Appends one finished episode, flushing so a killed process keeps it
    pub fn record(&mut self, episode: &JeopardyEpisode) -> io::Result<()> {
        let Some(file) = self.file.as_mut() else {
            return Ok(());
        };

        let line = serde_json::to_string(episode)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        writeln!(file, "{0}", line)?;

        file.flush()
    }

    /// Discards the log. Call this only once the real outfile is safely written,
    /// otherwise a failure there would leave nothing on disk at all.
    pub fn finish(self) -> io::Result<()> {
        let Some(path) = self.path else {
            return Ok(());
        };

        match fs::remove_file(path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{partial_path, ResumeLog};
    use crate::models::episode::JeopardyEpisode;
    use crate::models::question::Round;
    use crate::models::round::JeopardyRound;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Unique scratch path per test so they can run in parallel
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("j-archive-resume-{0}", name));

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir.join("results.json")
    }

    fn episode(id: u32) -> JeopardyEpisode {
        let round = |round| JeopardyRound {
            questions: Vec::new(),
            round,
        };

        JeopardyEpisode {
            air_date: Some("Thursday, November 17, 2022".to_string()),
            rounds: (
                round(Round::Jeopardy),
                round(Round::DoubleJeopardy),
                round(Round::FinalJeopardy),
            ),
            id,
        }
    }

    #[test]
    fn appends_partial_to_the_outfile_name() {
        assert_eq!(
            partial_path(Path::new("./out/results.json")),
            PathBuf::from("./out/results.json.partial")
        );
    }

    #[test]
    fn recovers_recorded_episodes_on_reopen() {
        let outfile = scratch("recovers");
        let path = outfile.to_str().unwrap();

        let mut log = ResumeLog::open(path, 1..10).unwrap();
        log.record(&episode(1)).unwrap();
        log.record(&episode(2)).unwrap();

        let mut resumed = ResumeLog::open(path, 1..10).unwrap();

        assert!(resumed.is_done(1));
        assert!(resumed.is_done(2));
        assert!(!resumed.is_done(3));

        let recovered = resumed.take_recovered();

        assert_eq!(recovered.len(), 2);
        assert_eq!(recovered[0].id, 1);
        // Taking the recovered episodes hands them over exactly once
        assert!(resumed.take_recovered().is_empty());
    }

    #[test]
    fn drops_a_line_torn_off_by_a_crash() {
        let outfile = scratch("torn");
        let path = outfile.to_str().unwrap();

        let mut log = ResumeLog::open(path, 1..10).unwrap();
        log.record(&episode(1)).unwrap();

        // Simulate a process killed mid-write
        let partial = partial_path(&outfile);
        let mut contents = fs::read_to_string(&partial).unwrap();
        contents.push_str("{\"air_date\":\"Monday\",\"rounds\":[{\"questi");
        fs::write(&partial, contents).unwrap();

        let mut resumed = ResumeLog::open(path, 1..10).unwrap();

        assert!(resumed.is_done(1));
        assert_eq!(resumed.take_recovered().len(), 1);
    }

    #[test]
    fn ignores_a_log_left_by_a_crawl_of_another_range() {
        let outfile = scratch("range");
        let path = outfile.to_str().unwrap();

        let mut log = ResumeLog::open(path, 1..10).unwrap();
        log.record(&episode(5)).unwrap();

        let mut resumed = ResumeLog::open(path, 100..110).unwrap();

        assert!(!resumed.is_done(5));
        assert!(resumed.take_recovered().is_empty());
    }

    #[test]
    fn keeps_one_copy_of_an_episode_recorded_twice() {
        let outfile = scratch("dupes");
        let path = outfile.to_str().unwrap();

        let mut log = ResumeLog::open(path, 1..10).unwrap();
        log.record(&episode(3)).unwrap();
        log.record(&episode(3)).unwrap();

        let mut resumed = ResumeLog::open(path, 1..10).unwrap();

        assert_eq!(resumed.take_recovered().len(), 1);
    }

    #[test]
    fn finish_removes_the_log() {
        let outfile = scratch("finish");
        let path = outfile.to_str().unwrap();

        let mut log = ResumeLog::open(path, 1..10).unwrap();
        log.record(&episode(1)).unwrap();

        let partial = partial_path(&outfile);
        assert!(partial.exists());

        log.finish().unwrap();
        assert!(!partial.exists());
    }

    #[test]
    fn a_disabled_log_records_nothing() {
        let mut log = ResumeLog::disabled();

        log.record(&episode(1)).unwrap();

        assert!(!log.is_done(1));
        assert!(log.take_recovered().is_empty());
        log.finish().unwrap();
    }
}
