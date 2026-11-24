// cyps - clean your projects.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version. See <http://www.gnu.org/licenses/>.

use std::env;
use std::fmt;
use std::path::Path;
use std::process::Command;

use clap::{Parser, ValueEnum};
use walkdir::WalkDir;

/// An action runs `command` in every directory that contains `name`.
struct Action {
    trigger_kind: Kind,
    trigger_name: &'static str,
    impact: ActionImpact,
    command: &'static str,
}

enum Kind {
    Dir,
    File,
}

/// How destructive an action is. Declaration order defines the ordering: an
/// action runs only when its impact is at most the requested impact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ActionImpact {
    /// Safe, in-place maintenance such as pack compaction.
    Compaction,
    /// Rebuildable caches and build output.
    CacheClearing,
    /// Generated dependencies and environments that must be re-downloaded.
    DerivedDestruction,
}

impl fmt::Display for ActionImpact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self
            .to_possible_value()
            .expect("every impact variant has a CLI value");
        f.write_str(value.get_name())
    }
}

/// Recursively clean projects below the current directory.
#[derive(Debug, Parser)]
struct Args {
    /// Recurse at most DEPTH levels below the current directory
    #[arg(short = 'd', long, value_name = "DEPTH", default_value_t = 2)]
    max_depth: usize,

    /// Only run actions up to this impact level
    #[arg(long, value_enum, default_value_t = ActionImpact::CacheClearing)]
    impact: ActionImpact,
}

// Edit this list to configure which clean-up commands run where.
const ACTIONS: &[Action] = &[
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".git",
        impact: ActionImpact::Compaction,
        command: "git gc",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".pytest_cache",
        impact: ActionImpact::CacheClearing,
        command: "rm -rf .pytest_cache",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".mypy_cache",
        impact: ActionImpact::CacheClearing,
        command: "rm -rf .mypy_cache",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".ruff_cache",
        impact: ActionImpact::CacheClearing,
        command: "rm -rf .ruff_cache",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".tox",
        impact: ActionImpact::DerivedDestruction,
        command: "rm -rf .tox",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".nox",
        impact: ActionImpact::DerivedDestruction,
        command: "rm -rf .nox",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: "__pycache__",
        impact: ActionImpact::CacheClearing,
        command: "rm -rf __pycache__",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "pom.xml",
        impact: ActionImpact::CacheClearing,
        command: "mvn clean",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "gradlew",
        impact: ActionImpact::CacheClearing,
        command: "./gradlew clean",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "build.sbt",
        impact: ActionImpact::CacheClearing,
        command: "sbt clean",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "project.clj",
        impact: ActionImpact::CacheClearing,
        command: "lein clean",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "ccResolutions",
        impact: ActionImpact::DerivedDestruction,
        command: "ccbuild distclean",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "Setup.hs",
        impact: ActionImpact::CacheClearing,
        command: "cabal clean",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".stack-work",
        impact: ActionImpact::CacheClearing,
        command: "rm -rf .stack-work",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "rebar.config",
        impact: ActionImpact::CacheClearing,
        command: "rebar3 clean",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "package.json",
        impact: ActionImpact::DerivedDestruction,
        command: "rm -rf node_modules",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".turbo",
        impact: ActionImpact::CacheClearing,
        command: "rm -rf .turbo",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".parcel-cache",
        impact: ActionImpact::CacheClearing,
        command: "rm -rf .parcel-cache",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "Cargo.toml",
        impact: ActionImpact::CacheClearing,
        command: "cargo clean",
    },
    Action {
        trigger_kind: Kind::File,
        trigger_name: "uv.lock",
        impact: ActionImpact::DerivedDestruction,
        command: "rm -rf .venv",
    },
    Action {
        trigger_kind: Kind::Dir,
        trigger_name: ".terraform",
        impact: ActionImpact::DerivedDestruction,
        command: "rm -rf .terraform",
    },
];

fn contains(action: &Action, path: &Path) -> bool {
    let target = path.join(action.trigger_name);
    match action.trigger_kind {
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
    let args = Args::parse();

    for entry in WalkDir::new(".").max_depth(args.max_depth) {
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
            if action.impact <= args.impact && contains(action, path) {
                run(action.command, path);
            }
        }
    }
    println!("Done");
}
