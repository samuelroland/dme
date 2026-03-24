<div align="center">
<img src="https://raw.githubusercontent.com/samuelroland/dme/main/docs/logo/logo.svg" alt="DME logo" height="128"/>
</div>

# Delightful Markdown Experience
>  What if the whole experience with Markdown was as delightful as the redaction ? Let's redefine the export, navigation and code highlighting experience too.

## Why
Because it's a pain to export Markdown as PDF and have a nice style. Because it's a pain to write pure CSS to customize dozens of little things by hand. Because the syntax highlighting is very fragile with large parts of uncolored code. Because switching between different files is slow, you shouldn't need to open a file explorer to find your notes. You shouldn' need to remember where they are or which section talked about a specific note. You should be able to search and find the information directly in the previewer.

## Installation

You need Git and the Rust toolchain 1.87 ([See installation via Rustup](https://rustup.rs/)). You also need a C compiler so DME can compile Tree-sitter grammars.

```sh
git clone https://github.com/samuelroland/dme.git
cd dme
```

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


### The desktop app
1. Make sure you have the Tauri prequisites so all build dependencies will be present: [Tauri prequisites](https://tauri.app/start/prerequisites/)
1. The frontend is built using [NodeJS v22+](https://nodejs.org) and [Pnpm 10+](https://pnpm.io/), make sure you have both of them
#### Running the desktop app for development
Just run
```sh
cd app
pnpm install
pnpm tauri dev
```

#### Building the desktop app for production
Note: this is far ready to be fully usable for now, but if you want to install DME globally, here are the instructions.

**WARNING: This is working mostly on Linux, installers for Windows are generated as `.msi` and for MacOS as `.dmg` but some features have not been tested or do not work**
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

### License
The DME project is released under the [MIT license](LICENSE).

---

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
│       ├── build.rs
│       ├── Cargo.toml
│       ├── icon
│       ├── src
│       │   ├── commands // Tauri commands by features, exposed for the frontend
│       │   │   ├── grammars.rs
│       │   │   ├── home.rs
│       │   │   ├── preview.rs
│       │   │   └── search.rs
│       │   ├── lib.rs
│       │   └── main.rs
│       └── tauri.conf.json
├── docs
├─ README.md
```

## Common errors

### In tests
Note: **try to never change the `PATH`** variable in tests, it will affects other tests as well as they run in parallel !!! It works with a single test but not when
We only do it at one exception in `setup.rs` for test `test_large_markdown_preview_with_codes_gives_same_result` because this is an integration test that is alone and always run after the unit tests.

### In Tauri backend

#### its trait bounds were not satisfied


```rust
pub enum InstalledStatus {
    NotInstalled,
    Installing,
    Installed,
}

pub struct GrammarState {
    id: String,
    link: String,
    status: InstalledStatus,
}

#[tauri::command]
pub fn get_grammars_list() -> Result<Vec<GrammarState>, String> {
```
```
error[E0599]: the method `blocking_kind` exists for reference `&Result<Vec<GrammarState>, String>`, but its trait bounds were not satisfied
   --> src/commands/grammars.rs:19:1
    |
19  |   #[tauri::command]
    |   ^^^^^^^^^^^^^^^^^ method cannot be called on `&Result<Vec<GrammarState>, String>` due to unsatisfied trait bounds
```

It misses the `#[derive(Serialize)]` on the 2 structs to be able to serialize them!

#### slow `invoke` call from frontend is blocking the browser thread
Even without using `await` before the `invoke("action", ...)` call, it seems to block the thread if the action is slow. To fix that and get a real async system from the frontend point of view, we can just turn on `async` command.

Instead of this definition of the command
```rust
#[tauri::command]
pub fn install_grammar(id: &str) -> Result<(), String> {
```
define it as `async`
```rust
#[tauri::command(async)]
pub fn install_grammar(id: &str) -> Result<(), String> {
```
