#![warn(rust_2018_idioms)]

#[macro_use]
extern crate prettytable;

use prettytable::Table;
use std::io::prelude::*;
use std::{env, fs::File, fs::OpenOptions, io::BufReader, path::PathBuf};
use structopt::StructOpt;

mod repo_overrides;
mod workspace;

#[derive(StructOpt, Debug)]
#[structopt(name = "bzlcfg", about = "Manages bazel configuration")]
enum CLI {
    #[structopt(name = "link", about = "Manages local repository overrides")]
    Link(Link),
}

#[derive(StructOpt, Debug)]
enum Link {
    #[structopt(
        name = "add",
        about = "Adds the current workspace to global repository overrides"
    )]
    Add {
        #[structopt(parse(from_os_str), default_value = ".")]
        workspace: PathBuf,
    },
    #[structopt(name = "list", about = "Lists current repository overrides")]
    List,
    #[structopt(
        name = "remove",
        about = "Removes a workspace from global repository overrides"
    )]
    Remove { name: String },
}

impl Link {
    fn run(&mut self, mut bazelrc: &mut File) -> std::io::Result<()> {
        match self {
            Self::Add { workspace } => add_link(workspace, &mut bazelrc),
            Self::List => {
                let mut table = Table::new();
                let format = prettytable::format::FormatBuilder::new()
                    .column_separator('|')
                    .separators(
                        &[
                            prettytable::format::LinePosition::Top,
                            prettytable::format::LinePosition::Title,
                        ],
                        prettytable::format::LineSeparator::default(),
                    )
                    .padding(1, 1)
                    .build();
                table.set_format(format);
                table.set_titles(row![bFg->"Workspace Name", bFg->"Workspace Path"]);

                for link in repo_overrides::get_all(&mut BufReader::new(bazelrc)) {
                    table.add_row(row![link.workspace, link.path]);
                }

                table.printstd();
                Ok(())
            }
            Self::Remove { name: _ } => unimplemented!(),
        }
    }
}

fn add_link(path: &mut PathBuf, mut bazelrc: &mut File) -> std::io::Result<()> {
    let (workspace_file, mut workspace_path) =
        workspace::find_up(&mut path.canonicalize().unwrap())
            .expect("Could not read WORKSPACE file");
    let workspace_name = workspace::name(&mut BufReader::new(&workspace_file))
        .expect("Could not parse name from WORKSPACE file");

    // Get the workspace root directory
    workspace_path.pop();

    println!(
        "Adding a local override for {} to your global bazel config",
        workspace_name
    );

    // Handle existing links
    if let Some(existing) =
        repo_overrides::get_by_workspace(&mut BufReader::new(&mut bazelrc), &workspace_name)
    {
        if existing.path == workspace_path.to_str().unwrap() {
            return Ok(());
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "a link to a different path already exists for this workspace",
        ));
    }

    writeln!(
        bazelrc,
        "build --override_repository={}={}",
        workspace_name,
        workspace_path.to_str().unwrap()
    )
}

fn main() -> std::io::Result<()> {
    let home = env::var("HOME").expect("$HOME is not set");
    let config_path = PathBuf::from(home).join(".bazelrc");
    let mut config_file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(config_path)
        .expect("Could not read $HOME/.bazelrc");

    match CLI::from_args() {
        CLI::Link(mut cmd) => cmd.run(&mut config_file),
    }
}
