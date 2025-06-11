#import "@preview/typslides:1.2.3": * // https://github.com/manjavacas/typslides
#import "@preview/tablex:0.0.9": tablex

// Project configuration
#show: typslides.with(
  ratio: "16-9",
  theme: "yelly",
)
#set text(font: "Cantarell")
#let figure = figure.with(
  kind: "image",
  supplement: none,
) // disable prefix in captions

#blank-slide[
  #align(center, [
  #image("logo/logo.svg", height: 5em)
    ```sh cargo build --release```

  #text(weight: "bold", size: 1.2em, fill: black)[Delightful Markdown Experience]
  ])
]

#slide(title: "Demo")[ ]
// open from start menu
// view progress indexation
// open search to find hpc report by path
// other search by heading, see streaming
// see htop threads usage
// see colored snippets
//

#slide(title: "Global architecture")[ ]
list of modules
core library
tauri in rust
vuejs frontend

#slide(title: "Search strategy")[ ]
- Split the data
- Prepare shared ressource
- Computation
- Update shared ressource

#slide(title: "Search strategy")[ ]
```rust
for chunk in all_paths.chunks(chunk_size) {
    let chunk = chunk.to_vec(); // copy chunk
    let title_map = Arc::clone(&self.title_map);

    thread::spawn(move || {
        for path in chunk {
            let titles = DiskResearcher::extract_markdown_titles(&path);
            {
                let mut map = title_map.lock().unwrap();
                for title in titles {
                    map.entry(title).or_default().push(path.clone())
                }
            }
```
#slide(title: "Syntax highlighting")[ ]
- Tree-Sitter vite fait
- Grammars installation
- Highlighting process
- Exemple paradigme application

#slide(title: "Future of DME ?")[ ]
- Making syntax highlighting parallel
- Making grammars installation parallel
- Making a full text search

#slide(title: "Our experience with the paradigm")[ ]
- Avoided thousands of possible errors
- Hard to think about advanced memory references
- No memory crash at runtime

#slide(title: "Our experience with Rust")[ ]
- The standard library
- Tree-Sitter library
- Unit and integration testing
- Type expressiveness
- Be forced to manage errors
- Liked the functionnal part of Rust
- Compilers contextual errors
- Proposed fixes and refactoring

