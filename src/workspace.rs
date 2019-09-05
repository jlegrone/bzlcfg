use recap::Recap;
use serde::Deserialize;
use std::io::prelude::*;
use std::{fs, fs::File, path::PathBuf};

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"workspace\(name = "(?P<name>\S+)"\)"#)]
struct BazelWorkspace {
    name: String,
}

pub fn find_up(path: &mut PathBuf) -> Result<(File, PathBuf), std::io::Error> {
    let workspace_path = fs::read_dir(&path)?
        .filter_map(Result::ok)
        .map(|item| item.path())
        .filter(|item| item.is_file())
        .find(|file| file.file_stem().unwrap().to_str().unwrap() == "WORKSPACE");

    match workspace_path {
        Some(path) => File::open(&path).map(|file| (file, path)),
        None => {
            if path.pop() {
                find_up(path)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "could not find WORKSPACE",
                ))
            }
        }
    }
}

pub fn name<R: BufRead>(workspace: &mut R) -> Option<String> {
    let workspace: Option<BazelWorkspace> = workspace
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.parse())
        .filter_map(Result::ok)
        .find(|_| true);
    match workspace {
        Some(BazelWorkspace { name }) => Some(name),
        _ => None,
    }
}
