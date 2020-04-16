use clap::Clap;
use std::error::Error;
use walkdir::WalkDir;

mod dockerignore;

use dockerignore::DockerIgnore;

#[derive(Clap, Debug)]
struct Opts {
    directory: String,
    #[clap(short = "a", long = "absolute")]
    absolute_paths: bool,
    #[clap(long = "dockerignore")]
    dockerignore_path: Option<String>,
}

fn iterate_entries<'a>(
    root_directory: &'a str,
    dockerignore: &'a DockerIgnore,
) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> + 'a {
    WalkDir::new(root_directory)
        .sort_by(|a, b| a.path().cmp(b.path()))
        .into_iter()
        .filter_entry( move |entry| {
            !dockerignore.check_path_ignored(
                &entry
                    .path()
                    .strip_prefix(&root_directory)
                    .expect("prefix error"),
            )
        })
}

fn go(opts: &Opts) -> Result<(), Box<dyn Error>> {
    let root_directory = &opts.directory;
    let dockerignore_path = match &opts.dockerignore_path {
        None => format!("{}/.dockerignore", root_directory),
        Some(val) => val.to_owned(),
    };
    let dockerignore = DockerIgnore::read(dockerignore_path)?;
    for entry in iterate_entries(&root_directory, &dockerignore) {
        match entry {
            Ok(entry) => {
                if entry.metadata()?.is_file() {
                    let path = if opts.absolute_paths {
                        entry.path()
                    } else {
                        entry.path().strip_prefix(&root_directory)?
                    };
                    println!("{}", path.display());
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
