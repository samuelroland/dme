<div align="center">
<img src="https://raw.githubusercontent.com/samuelroland/dme/main/docs/logo/logo.svg" alt="DME logo" height="128"/>
</div>

# Delightful Markdown Experience
>  What if the whole experience with Markdown was as delightful as the redaction ? Let's redefine the export, navigation and code highlighting experience too.

## Why

## Installation

You need Git, the Rust toolchain ([See installation via Rustup](https://rustup.rs/))

### Developing on the core library only
```sh
cd app/core
cargo build
```

Look at `src/lib.rs` for now and generate the doc with `cargo doc` to understand more about the available data structures.

How to run tests
```sh
cargo test
```

How to run tests and include ignored tests (they are marked as `#[ignored]` because they need network access or are slow to run)
```sh
cargo test -- --include-ignored
```

Note: **never change the `PATH`** variable in tests, it will affects other tests as well!

### Running the desktop app for development
1. Make sure you have the Tauri prequisites, the desktop app will not build otherwise: [Tauri prequisites](https://tauri.app/start/prerequisites/)
1. The frontend is built using [pnpm](https://pnpm.io/), make sure you have that too
    ```sh
    cd app
    pnpm install
    pnpm tauri dev
    ```

### Building the desktop app for production
Note: this is far ready to be fully usable for now, but if you want to install DME globally, here are the instructions.

**WARNING: this only has been tested for Linux, installers for windows are generated as `.msi` but they have not been tested**
1. Just run this
    ```sh
    cd app
    pnpm install
    pnpm tauri build && pnpm tauri bundle
    ```
1. On Fedora
   ```sh
   sudo dnf install src-tauri/target/release/bundle/deb/dme_0.1.0_amd64.deb
   ```
1. Or on Ubuntu
   ```sh
   sudo apt install /src-tauri/target/release/bundle/rpm/dme-0.1.0-1.x86_64.rpm
   ```

## Project structure
This is a cleaned overview of the project structure with a few comments.
```
dme
├── app
│   ├── core // The core library
│   │   ├── Cargo.toml
│   │   ├── docs.md
│   │   ├── README.md
│   │   ├── src
│   │   │   ├── export
│   │   │   │   ├── chromium.rs
│   │   │   │   └── export.rs
│   │   │   ├── lib.rs // Where markdown_to_highlighted_html() lives
│   │   │   ├── preview
│   │   │   │   ├── comrak.rs
│   │   │   │   ├── preview.rs
│   │   │   │   ├── proposed_grammars.rs
│   │   │   │   ├── tree_sitter_grammars.rs // Grammars management
│   │   │   │   └── tree_sitter_highlight.rs // Highlighting code with Tree-sitter
│   │   │   ├── search
│   │   │   │   ├── disk.rs
│   │   │   │   └── search.rs
│   │   │   ├── theming
│   │   │   │   ├── error.rs
│   │   │   │   ├── helix.rs // Helix TOML theme loader
│   │   │   │   ├── renderer.rs
│   │   │   │   └── theme.rs
│   │   │   ├── util
│   │   │   │   ├── git.rs // Git CLI wrapper
│   │   │   │   └── setup.rs
│   │   │   └── util.rs
│   │   └── tests // Integration tests
│   │       ├── common
│   │       │   └── regression.rs
│   │       ├── large_preview.rs // Integration tests with large files converted to HTML
│   │       ├── large_search.rs // Integration tests with large Markdown dataset of MDN
│   │       ├── README.md
│   │       └── reference // Reference files as HTML
│   ├── package.json
│   ├── src
│   │   ├── App.vue
│   │   ├── Home.vue
│   │   ├── main.css
│   │   ├── main.ts
│   │   ├── Search.vue
│   │   ├── types.ts
│   ├── src-tauri
│   │   ├── build.rs
│   │   ├── Cargo.toml
│   │   ├── icon
│   │   ├── src
│   │   │   ├── lib.rs
│   │   │   └── main.rs
│   │   └── tauri.conf.json
├── docs
├─ README.md
```
