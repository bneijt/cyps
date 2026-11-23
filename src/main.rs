// cyps - clean your projects.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version. See <http://www.gnu.org/licenses/>.

use std::env;
use std::path::Path;
use std::process::Command;

use clap::{Arg, Command as ClapCommand, value_parser};
use walkdir::WalkDir;

/// An action runs `command` in every directory that contains `name`.
struct Action {
    kind: Kind,
    name: &'static str,
    command: &'static str,
}

enum Kind {
    Dir,
    File,
}

// Edit this list to configure which clean-up commands run where.
#[rustfmt::skip]
const ACTIONS: &[Action] = &[
    Action { kind: Kind::Dir,  name: ".git",          command: "git gc" },
    Action { kind: Kind::Dir,  name: ".pytest_cache", command: "rm -rf .pytest_cache" },
    Action { kind: Kind::Dir,  name: ".mypy_cache",   command: "rm -rf .mypy_cache" },

    Action { kind: Kind::File, name: "pom.xml",       command: "mvn clean" },
    Action { kind: Kind::File, name: "ccResolutions", command: "ccbuild distclean" },
    Action { kind: Kind::File, name: "Setup.hs",      command: "cabal clean" },
    Action { kind: Kind::File, name: "package.json",  command: "rm -rf node_modules" },
    Action { kind: Kind::File, name: "Cargo.toml",    command: "cargo clean" },
    Action { kind: Kind::File, name: "uv.lock",       command: "rm -rf .venv dist" },
];

fn configured_max_depth() -> usize {
    let matches = ClapCommand::new("cyps")
        .about("Recursively clean projects below the current directory")
        .arg(
            Arg::new("max-depth")
                .short('d')
                .long("max-depth")
                .value_name("DEPTH")
                .help("Recurse at most DEPTH levels below the current directory")
                .value_parser(value_parser!(usize))
                .default_value("2"),
        )
        .get_matches();

    *matches
        .get_one::<usize>("max-depth")
        .expect("max-depth has a default value")
}

fn contains(action: &Action, path: &Path) -> bool {
    let target = path.join(action.name);
    match action.kind {
        Kind::Dir => target.is_dir(),
        Kind::File => target.is_file(),
    }
}

fn run(command: &str, path: &Path) {
    let old_pwd = env::current_dir().expect("cannot read current directory");
    env::set_current_dir(path).expect("cannot enter directory");
    let cur_pwd = env::current_dir().expect("cannot read current directory");
    println!("[{}]", cur_pwd.display());
    let _ = Command::new("/bin/sh").arg("-c").arg(command).status();
    env::set_current_dir(old_pwd).expect("cannot restore directory");
}

fn main() {
    for entry in WalkDir::new(".").max_depth(configured_max_depth()) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                // Commands such as `cabal clean` can remove directories that the
                // walk had already buffered; that is not worth reporting.
                let removed = err
                    .io_error()
                    .is_some_and(|err| err.kind() == std::io::ErrorKind::NotFound);
                if !removed {
                    eprintln!("cyps: {err}");
                }
                continue;
            }
        };

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        for action in ACTIONS {
            if contains(action, path) {
                run(action.command, path);
            }
        }
    }
    println!("Done");
}
