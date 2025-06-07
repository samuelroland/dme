# dme_core
### The core library of DME, backing up the CLI and Tauri app

How to run tests
```sh
cargo test -- --include-ignored --test-threads 1
```

How to run tests and include ignored tests (because need network access or are slow)
```sh
cargo test -- --include-ignored --test-threads 1
```
Note: they must be used in parallel because some operations with Git don't work in 


How to run tests only ignored tests
```sh
cargo test -- --ignored --test-threads 1
```
