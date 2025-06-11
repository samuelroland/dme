
== Architecture

== Implementation
=== Indexing the disk
=== Searching with the index

== Syntax highlighting

- Tree-Sitter vite fait
- Grammars installation
- Highlighting process
- Exemple paradigme application


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



