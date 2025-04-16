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
    - PDF Export
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
]

#slide(title: "Why memory safety is big deal")[

]
graphique de 70% de microsot CVe
-> mentionner garbage collector + manual

#slide(title: "What's the solution")[

]
Rust new paradigms

2 concepts

+ section why...

#slide(title: "concurrency tricks")[

]

#slide(title: "Recap des 3 sujets")[

]

