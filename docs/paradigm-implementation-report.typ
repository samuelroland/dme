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

We splitted our application in multiple crate. First we have a frontend crate, under src-tauri
//TODO SAM Explain what it does

Then we have our backend crate. This crate itself is splitted in multiple module, each having their own responsabilites.

- preview responsible for generating the html to render on the front side
- theming responsible for syntax higlithing
- search respnsible for makdown indexing


== Implementation

The search crate contains two core functionallity:
- The indexation of markdown title on disk
- The search inside the built

An important point is that the indexing process is asynchronous, running in separated threads. The reasons for this is
avoid blocking UI / caller on a start call. Instead, the DiskResearcher make available a Progress struc to know how
much of the markdowns are treated.
=== Indexing the disk

This is the base algorithm of the indexation of markdown title on disk
```
chunks = split the markdown found on disk
for each chunks
    start a thread
        get makrdown content
        extract title base on content
        lock
        write data to shared map of title
        unlock
    end thread
end for
```
=== Searching with the index

This is the base algorithm of the search inside the title map build by the indexing process
```
chunks = split the title map
for each chunks
    start a thread
        evaluate how close the title is to the query
        lock
        write proximity to shared map of results
        unlock
    end thread
end for

search for markdown path matching the query
join and sort results
```

The process of evaluating the proximity of the titles/path and the query use an external crate: nucleo_matcher.
Each results is attributed a score, with the path receiving a 1.3 multiplicatore compared to title to prioritize them.
Those results are then sorted, and a subset is returned.

To ensure the relevency of the resulsts, the sub is calculated as followed:

```
get the highest match proximity-wise
define the lowerbound of accepted matches:
   tale 3/4 of the maximum

filter all matches to keep the one above lowerbound
return the results
```

An important note is that even if we limit the results to let's say 10, we might still get fewer results, if the were deemed
not relevant enough.

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

Then standard library offerd quiet a lot of advanced feature that were really helpful, such as Arc or RwLock.
They were easy to use and well-documented
- Tree-Sitter library
The Tree-Sitter library was the complete opposite of the standard library of rust, there was hardly any documentation,
often requiering to read the code to understand how it worked.
- Unit and integration testing
We were able to write a lot of Unit tests, and even though are program was highly concurrent we did not suffer any side
effect linked to concurrency. Our tests were deterministic.
- Type expressiveness
- Be forced to manage errors
The fact that rust force direct management of error had a lot of cost during the coding process.
Fortunetly this cost was leverage against less runtime issues.

- Liked the functionnal part of Rust
- Compilers contextual errors
- Proposed fixes and refactoring

loader local ref error mess


structure arc mutex dindex

pourquoi ca dans la recherche
            let chunk = chunk.to_vec(); // copy chunk

surtout lié au join skippé


exemple du verygood.rs
```rust

TSH_CACHE: Lazy\<RwLock<HashMap>>

/// A theme defining colors and modifiers to be used for syntax highlighting.
pub struct Theme<'a> {
  //...
    pub(crate) supported_highlight_names: &'a [&'a str],
}


/// HTML syntax highlighting renderer.
pub struct Renderer<'a> {
    theme: &'a theme::Theme<'a>,
}todo install instructions

```

== Future of DME ?
- Making syntax highlighting parallel
- Making grammars installation parallel
- Making a full text search

== Opportunities to dive deeper in the paradigm

TODO
- il manquait une conclusion/synthèse un peu et peut-être des idées de comment cette recherche pouvait être continuée dans un autre travail.



