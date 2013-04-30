Clean your projects (cyps)
--------------------------
_A programmers clean-up utility_

Recursively goes through directories and executes clean commands (`git gc`, `maven clean`, `cabal clean`, etc.)

The single tools is good enough for me, but I'd be happy to pull in changes for the following features:

- Filter actions based on which tools you have, so `cyps` will not try to run `mvn clean` when you don't have `mvn`.
- More clean up jobs
- Refactoring towards a more DSL like approach
- Support for `--help`


Configure and install
---------------------
The command is a single Haskell script that needs to be run by `runghc`.

To configure the command, open in any text editor and edit the `actions` value. This is a listing of `(condition, command)` tuples.

    actions :: [Action]
    actions = [
        (containsDirectory ".git", run "git gc"),
        (containsFile "pom.xml", run "mvn clean"),
        (containsFile "ccResolutions", run "ccbuild distclean"),
        (containsFile "Setup.hs", run "cabal clean")
        ]

The first element of the list can be read as: if a directory `containsDirectory` named `.git`, then `run` the command `git gc`.
