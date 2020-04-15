use clap::Clap;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Clap, Debug)]
struct Opts {
    directory: String,
    #[clap(short = "a", long = "absolute")]
    absolute_paths: bool,
    #[clap(long = "dockerignore")]
    dockerignore_path: Option<String>,
}

#[derive(Debug)]
struct DockerIgnoreLine {
    string: String,
    lineno: usize,
    is_comment: bool,
    is_negate: bool,
    pattern: Result<glob::Pattern, glob::PatternError>,
}

#[derive(Debug)]
struct DockerIgnore {
    lines: Vec<DockerIgnoreLine>,
}

impl DockerIgnore {
    fn read<P>(filename: P) -> Result<DockerIgnore, io::Error>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        let lines = io::BufReader::new(file).lines();
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

    fn check_path_ignored(self: &Self, path: &Path) -> bool {
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

fn go(opts: &Opts) -> Result<(), Box<dyn Error>> {
    let root_directory = &opts.directory;
    let dockerignore_path = match &opts.dockerignore_path {
        None => format!("{}/.dockerignore", root_directory),
        Some(val) => val.to_owned(),
    };
    let dockerignore = DockerIgnore::read(dockerignore_path)?;
    for entry in WalkDir::new(root_directory)
        .sort_by(|a, b| a.path().cmp(b.path()))
        .into_iter()
        .filter_entry(|entry| {
            !dockerignore.check_path_ignored(
                &entry
                    .path()
                    .strip_prefix(&root_directory)
                    .expect("prefix error"),
            )
        })
    {
        match entry {
            Ok(entry) => {
                if entry.metadata()?.is_file() {
                    println!(
                        "{}",
                        if opts.absolute_paths {
                            entry.path()
                        } else {
                            entry.path().strip_prefix(&root_directory)?
                        }
                        .display()
                    );
                }
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    }
    Ok(())
}

fn main() {
    let opts: Opts = Opts::parse();
    go(&opts).expect("internal error");
}
