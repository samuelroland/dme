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
    ```sh cargo test```#linebreak()
    ```sh cargo build --release```

  #text(weight: "bold", size: 1.2em, fill: black)[Ready to enter the Delightful Markdown Experience ?]
  ])
]


#title-slide[
  Demo !
]
// open from start menu
// view progress indexation
// open search to find hpc report by path
// other search by heading, see streaming
// see htop threads usage
// see colored snippets
//

#slide(title: "Global architecture")[
#image("schemas/architecture.png")
]

#slide(title: "Search strategy")[
#grid(
  columns: (2fr, 3fr),
[
- Split the data
- Prepare shared ressource
- Computation
- Update shared ressource
    ],
    [
  #image("schemas/diskresearcher.png")
// todo update schema with rwlock
  ]
)
]

// todo update code
#slide(title: "Search strategy")[
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
]

#slide(title: "Syntax highlighting with Tree-sitter")[
#text(size: 0.56em)[

#grid(
  columns: (2fr, 2fr, 2fr),
[
  `tree-sitter.json`
```json
{
  "grammars": [
    {
      "name": "css",
      "camelcase": "CSS",
      "scope": "source.css",
      "path": ".",
      "file-types": [ "css" ],
      "highlights": "queries/highlights.scm",
      "injection-regex": "^css$"
    }
  ],
  // ..
}
```
],
[

`grammar.js`

```js
...
 rules: {
    stylesheet: $ => repeat($._top_level_item),

    // Statements
    import_statement: $ => seq(
      '@import',
      $._value,
      sep(',', $._query),
      ';',
    ),
...
```


],
[

C parser
```sh
src> tree
├── grammar.json
├── node-types.json
├── parser.c
├── scanner.c
└── tree_sitter
    ├── alloc.h
    ├── array.h
    └── parser.h
```

],
[

#linebreak()
`queries/highlights.scm`
```scm
"~" @operator
">" @operator
"+" @operator
"-" @operator
...

(class_name) @property
(id_name) @property
(namespace_name) @property
```

  #linebreak()
`<span class="operator">+</span>`

], [

`catpuccin-latte.toml`
```toml
"constant" = "peach"
"constant.character" = "teal"
"constant.character.escape" = "pink"

"string" = "green"
...

[palette]
rosewater = "#dc8a78"
flamingo = "#dd7878"
pink = "#ea76cb"
```],
  [
```sh
~/.local/share/tree-sitter-grammars>
tree-sitter-c
tree-sitter-cpp
tree-sitter-css
tree-sitter-csv
...
```
]
)
]
]

#slide(title: "Highlighting steps")[
#grid(
  columns: (1fr, 2fr),
  column-gutter: 32pt,
  [
- Download, compile, load
- Language configuration
- Load a highlighter
- Highlight some code
- Render HTML
- Include in bigger doc
  ],
[
#text(size: 0.7em)[

```rust
// git clone --depth 1 --single_branch
let only_latest_commits: Option<u32> = Some(1);
let mut args = vec!["clone", git_clone_url];
if let Some(count) = only_latest_commits {
    args.push("--depth");
    args.push(&count.to_string());
}
if single_branch {
    args.push("--single-branch");
}
```

```sh
args.push(&count.to_string());
           ^^^^^^^^^^^^^^^^^ - temporary value is freed at the end of this statement
           |
           creates a temporary value which is freed while still in use

args.push("--single-branch");
---- borrow later used here
```
]
]
)
]

#slide(title: "Future of DME ?")[
- Making syntax highlighting parallel
- Making grammars installation parallel
- Making a full text search
- Themes management
- PDF export
- Visual theme configuration
- More benchmarking and optimisations
]

#slide(title: "Our experience with the paradigm")[
- Avoided thousands of possible errors
- Hard to think about advanced memory references
- No memory crash at runtime
]

#slide(title: "Our experience with Rust")[
- The standard library
- Tree-Sitter library
- Unit and integration testing
- Type expressiveness
- Be forced to manage errors
- Liked the functionnal part of Rust
- Compilers contextual errors
- Proposed fixes and refactoring
]
