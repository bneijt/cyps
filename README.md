Clean your projects (cyps)
--------------------------
_A programmers clean-up utility_

Recursively walks the current directory up to `--max-depth` levels (default 2)
and executes clean commands (`git gc`, `mvn clean`, `cargo clean`, etc.) where
applicable.

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

    cyps [--max-depth DEPTH]

`--max-depth` (or `-d`) controls how deep the walk goes. The current directory
is depth 0, so the default of 2 visits the current directory, its children, and
its grandchildren. Use `-d 1` for the old behaviour of only visiting immediate
subdirectories. `cyps --help` shows the available options.
