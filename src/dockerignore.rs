use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::Path;

#[derive(Debug)]
pub struct DockerIgnoreLine {
    string: String,
    lineno: usize,
    is_comment: bool,
    is_negate: bool,
    pattern: Result<glob::Pattern, glob::PatternError>,
}

#[derive(Debug)]
pub struct DockerIgnore {
    lines: Vec<DockerIgnoreLine>,
}

impl DockerIgnore {
    pub fn read<P>(filename: P) -> Result<DockerIgnore, Error>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();
        let ents = lines
            .filter_map(|l| l.ok())
            .enumerate()
            .map(|(lineno, string)| {
                let is_comment = string.starts_with('#');
                let is_negate = string.starts_with('!');
                let pattern = glob::Pattern::new(if is_negate { &string[1..] } else { &string });
                DockerIgnoreLine {
                    string,
                    lineno: lineno + 1,
                    is_comment,
                    is_negate,
                    pattern,
                }
            });
        Ok(DockerIgnore {
            lines: ents.collect(),
        })
    }

    pub fn check_path_ignored(self: &Self, path: &Path) -> bool {
        let mut is_ignored: bool = false;
        for ie in &self.lines {
            if ie.is_comment {
                continue;
            }
            if ie.pattern.is_err() {
                continue;
            }
            if ie.pattern.as_ref().unwrap().matches_path(&path) {
                is_ignored = !ie.is_negate;
            }
        }
        is_ignored
    }
}
