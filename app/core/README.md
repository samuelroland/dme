# dme_core
### The core library of DME, backing up the CLI and Tauri app
## Use

### Code highlighting with Tree-Sitter
To highlight this piece of CSS
```css
#submit {
    color: blue !important;
}
```

You have to install a CSS Tree-Sitter grammar like that
```rust

```

## Develop
How to run tests
```sh
cargo test
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
