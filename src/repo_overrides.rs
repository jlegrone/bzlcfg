use recap::Recap;
use serde::Deserialize;
use std::io::prelude::*;

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"common --override_repository=(?P<workspace>\S+)=(?P<path>\S+)"#)]
pub struct OverrideRepository {
    pub workspace: String,
    pub path: String,
}

pub fn get_all<R: BufRead>(bazelrc: &mut R) -> Vec<OverrideRepository> {
    bazelrc
        .by_ref()
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.parse())
        .filter_map(Result::ok)
        .collect()
}

pub fn get_by_workspace<R: BufRead>(
    bazelrc: &mut R,
    workspace: &str,
) -> Option<OverrideRepository> {
    bazelrc
        .by_ref()
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.parse())
        .filter_map(Result::ok)
        .find(|link: &OverrideRepository| link.workspace == workspace)
}
