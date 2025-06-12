#set text(font: "Cantarell")
#show link: underline
#let figure = figure.with(
  kind: "image",
  supplement: none,
) // disable prefix in captions

#set page(
  margin: 30pt,
  numbering: "1",
  footer: align(
    center, 
    context(counter(page).display())
  )
)


#align(center)[
#text(size: 20pt)[= Ownership and lifetimes]
How Rust's unique features will help us develop a stable, fast and multi-threaded desktop app
#image("logo/logo.svg", height: 4em)
PLM - Paradigm analysis report 
]

#outline(
 title: "Table of Contents",
)

== Architecture

== Implementation
=== Indexing the disk
=== Searching with the index

== Syntax highlighting

- Tree-Sitter vite fait
- Grammars installation
- Highlighting process
- Exemple paradigme application



```rust
let only_latest_commits: Option<u32> = Some(1);
let mut args = vec!["clone", git_clone_url];
if let Some(count) = only_latest_commits {
    args.push("--depth");
    args.push(&count.to_string());
}
if single_branch {
    args.push("--single-branch");
}
let output = Self::run_git_cmd(&args, base_directory)?;
```

```sh
   Compiling dme-core v0.1.0
error[E0716]: temporary value dropped while borrowed
  --> src/util/git.rs:66:24
   |
66 |             args.push(&count.to_string());
   |                        ^^^^^^^^^^^^^^^^^ - temporary value is freed at the end of this statement
   |                        |
   |                        creates a temporary value which is freed while still in use
...
69 |             args.push("--single-branch");
   |             ---- borrow later used here
   |
   = note: consider using a `let` binding to create a longer lived value
```



== Our experience
=== Our experience with the paradigm
- Avoided thousands of possible errors
- Hard to think about advanced memory references
- No memory crash at runtime

=== Our experience with Rust
- The standard library
- Tree-Sitter library
- Unit and integration testing
- Type expressiveness
- Be forced to manage errors
- Liked the functionnal part of Rust
- Compilers contextual errors
- Proposed fixes and refactoring

#slide(title: "")[ ]

loader local ref error mess


structure arc mutex dindex

pourquoi ca dans la recherche
            let chunk = chunk.to_vec(); // copy chunk

surtout lié au join skippé


exemple du verygood.rs

TSH_CACHE: Lazy<RwLock<HashMap>>

/// A theme defining colors and modifiers to be used for syntax highlighting.
pub struct Theme<'a> {
  //...
    pub(crate) supported_highlight_names: &'a [&'a str],
}


/// HTML syntax highlighting renderer.
pub struct Renderer<'a> {
    theme: &'a theme::Theme<'a>,
}todo install instructions



== Future of DME ?
- Making syntax highlighting parallel
- Making grammars installation parallel
- Making a full text search

== Opportunities to dive deeper in the paradigm

TODO
- il manquait une conclusion/synthèse un peu et peut-être des idées de comment cette recherche pouvait être continuée dans un autre travail.



