# dme_core
### The core library of DME, backing up the CLI and Tauri app
## Use
Look at `src/lib.rs` for now and generate the doc with `cargo doc` to understand more about the available datastructures.

## Develop
How to run tests
```sh
cargo test
```

How to run tests and include ignored tests (they are marked as `#[ignored]` because they need network access or are slow to run)
```sh
cargo test -- --include-ignored
```

Note: **never change the `PATH`** variable in tests, it will affects other tests as well!
