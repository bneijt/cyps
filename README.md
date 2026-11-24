Clean your projects (cyps)
--------------------------
_A programmers clean-up utility_

Recursively walks the current directory up to `--max-depth` levels (default 2)
and executes clean commands (`git gc`, `mvn clean`, `cargo clean`, etc.) where
applicable. Directories are visited lazily, so clean-up starts before the whole
tree has been listed.

The single tool is good enough for me, but I'd be happy to pull in changes for
the following features:

- Filter actions based on which tools you have, so `cyps` will not try to run
  `mvn clean` when you don't have `mvn`.
- More clean up jobs
- Refactoring towards a more DSL like approach


Configure and install
---------------------
The command is a single Rust binary.

Build it with:

    cargo build --release

The binary is `target/release/cyps`. Copy it somewhere on your `PATH`, for
example:

    install -m 755 target/release/cyps ~/.local/bin/cyps

Run with:

    cyps [--max-depth DEPTH] [--impact LEVEL]

`--max-depth` (or `-d`) controls how deep the walk goes. The current directory
is depth 0, so the default of 2 visits the current directory, its children, and
its grandchildren. Use `-d 1` for the old behaviour of only visiting immediate
subdirectories. `cyps --help` shows the available options.

`--impact` (default `cache-clearing`) limits which actions run. Impact levels
are ordered from least to most destructive, and actions at or below the
selected level are executed:

| Level | Meaning | Examples |
|---|---|---|
| `compaction` | Safe in-place maintenance | `git gc` |
| `cache-clearing` | Rebuildable caches and build output | `mvn clean`, `cargo clean`, `cabal clean`, `__pycache__`, `.stack-work`, `.turbo` |
| `derived-destruction` | Generated dependencies that must be re-downloaded | `node_modules`, `.venv`, `.tox`, `.nox`, `.terraform`, `ccbuild distclean` |

For example, the default leaves `node_modules` and `.venv` alone. Use
`--impact derived-destruction` to remove those as well.


Actions
-------
To configure the command, edit the `ACTIONS` value in `src/main.rs`. Each entry
combines a trigger (a file or directory name), an impact level, and a shell
command.

| Trigger | Kind | Impact | Command |
|---|---|---|---|
| `.git` | directory | `compaction` | `git gc` |
| `.pytest_cache` | directory | `cache-clearing` | `rm -rf .pytest_cache` |
| `.mypy_cache` | directory | `cache-clearing` | `rm -rf .mypy_cache` |
| `.ruff_cache` | directory | `cache-clearing` | `rm -rf .ruff_cache` |
| `.tox` | directory | `derived-destruction` | `rm -rf .tox` |
| `.nox` | directory | `derived-destruction` | `rm -rf .nox` |
| `__pycache__` | directory | `cache-clearing` | `rm -rf __pycache__` |
| `pom.xml` | file | `cache-clearing` | `mvn clean` |
| `gradlew` | file | `cache-clearing` | `./gradlew clean` |
| `build.sbt` | file | `cache-clearing` | `sbt clean` |
| `project.clj` | file | `cache-clearing` | `lein clean` |
| `ccResolutions` | file | `derived-destruction` | `ccbuild distclean` |
| `Setup.hs` | file | `cache-clearing` | `cabal clean` |
| `.stack-work` | directory | `cache-clearing` | `rm -rf .stack-work` |
| `rebar.config` | file | `cache-clearing` | `rebar3 clean` |
| `package.json` | file | `derived-destruction` | `rm -rf node_modules` |
| `.turbo` | directory | `cache-clearing` | `rm -rf .turbo` |
| `.parcel-cache` | directory | `cache-clearing` | `rm -rf .parcel-cache` |
| `Cargo.toml` | file | `cache-clearing` | `cargo clean` |
| `uv.lock` | file | `derived-destruction` | `rm -rf .venv` |
| `.terraform` | directory | `derived-destruction` | `rm -rf .terraform` |

The first entry can be read as: if a directory contains a directory named
`.git`, run `git gc` in it when the requested impact is at least `compaction`.
