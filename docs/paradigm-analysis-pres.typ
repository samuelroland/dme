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
    ```sh cargo new the-delightful-markdown-experience```

  #text(weight: "bold", size: 2.2em, fill: black)[Ownership and lifetimes]

  #text(weight: "bold", size: 1.2em, fill: black)[How Rust's unique features will help us develop #linebreak() a stable, fast and multi-threaded desktop app]
  ])
]

#page(margin: 0pt)[
  #align(center)[#image("imgs/dme-poc.png", width: 101%)]
]

#slide(title: "The project")[

#tablex(
  columns:2,
  stroke: none,
  inset: 0pt,
  align: horizon,
  row-gutter: 0em,
  column-gutter: 0.3em,
)[
+ 3 main features
    - Research
    - Preview
    - PDF export
+ Maximum of parallelisation
+ Stability and low memory footprint

#grid(
  columns: (1fr, 1fr),
figure(
  image("imgs/tree-sitter-logo.png", width: 5em),
  caption: [TreeSitter],
),
figure(
  image("imgs/tauri-logo.png", width: 5em),
  caption: [Tauri],
)
)

][
#text(size: 15pt)[
Tree-Sitter generated HTML example
```html
<pre>
    <code class="language-c">
        <span class="keyword control repeat">for</span>
        <span class="punctuation bracket">(</span>
        <span class="type">sf_count_t</span>
        <span class="variable">i</span>
        <span class="operator">=</span>
        <span class="constant numeric">0</span>
        <span class="punctuation delimiter">;</span>
        <span class="variable">i</span>
        ...
    </code>
</pre>
```

]
]
]

#slide(title: "Basics of concurrency")[
#grid(
  columns: (2fr, 3fr),
text()[
      - 
    ],
text()[asdf ],
)
]

#slide(title: "Basics of memory management")[

#image("schemas/empty.png")
#place(
    bottom + right,
text(size: 17pt)[
```c
void print(int *toshow) {
  printf("%d", *toshow);
}
int main(void) {
  int a = 23;
  char *msg = "salut";
  char *ptr = malloc(sizeof(int)*SIZE);          
  print(&a);
  free(ptr);
}
```
])
]


#slide(title: "Basics of memory management")[

#image("schemas/filled.png")
#place(
    bottom + right,
text(size: 17pt)[
```c
void print(int *toshow) {
  printf("%d", *toshow);
}
int main(void) {
  int a = 23;
  char *msg = "salut";
  char *ptr = malloc(sizeof(int)*SIZE);          
  print(&a);
  free(ptr);
}
```
])

// -> mentionner garbage collector + manual
]

#slide(title: "Why memory safety is a big deal ?")[

#image("imgs/microsoft-cve-memory-portion-per-year.png", width: 36em)

#quote("~70% of the vulnerabilities addressed through a security update each year continue to be memory safety issues"). From #link("https://github.com/microsoft/MSRC-Security-Research/blob/master/presentations/2019_02_BlueHatIL/2019_01%20-%20BlueHatIL%20-%20Trends%2C%20challenge%2C%20and%20shifts%20in%20software%20vulnerability%20mitigation.pdf")[Microsoft presentation from 2019].

]

#slide(title: "What's the solution ?")[

== Rust new paradigms
- Advanced static analysis at compilation time
- In addition to a type and variable, each ressource has an *owner* and a *lifetime*
- Advanced smart pointers, traits and concurrency mecanisms
]

2 concepts

+ section why...

#slide(title: "concurrency tricks")[

]

#slide(title: "Recap of the magic")[
  Borrow checker enforced rules
  - Only one mutable reference at a time
  - Or several immutables references
  - References must always be valid
  - TODO
]

