# Integration tests

This folder contains integration tests. They have 4 mains goals
- make sure the external use the of the library is well designed
- make sure the high-level features are stable and working in a real life context (not searching among 20 Markdown files)
- avoid regressions during performance optimisations and general refactoring
- get confidence in the general stability of the library

## Strategy
We are not commiting big Markdown dataset to avoid bloating this repository. But we automate the generation or download of these files to make them reproducible.

We are actually commiting regressions tests output as reference to be able to run 

## How to run

It will run unit tests and then integration tests, they take at least 10secondes and can take much more.
```sh
cargo test
```
