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
#image("imgs/tree-sitter-codeblock-demo.png", width: 114%)
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
      - Multiple processes
      - Shared ressources
      - Critical section
    ],
    [
    #image("schemas/concurrency.png")
  ]
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
- Invented at Mozilla, released 1.0 in 2015, most loved programming language Stackoverflow survey from 2016
- Advanced static analysis at compilation time
- In addition to a type and variable, each ressource has an *owner* and a *lifetime*
- Advanced smart pointers, traits and concurrency mecanisms
]

#slide(title: "Lifetimes and Ownership")[
//usful example ?
```c
// library.h
void save_file(float* buffer, char* filename);

// main.c
float* buffer = malloc(SIZE * sizeof(float));
char* filename = "test.txt";
save_file(buffer, filename);
free(buffer); // changed ? need to be freed ???
```
]

#slide(title: "Ownership")[
  #grid(columns: 2,
  row-gutter: 45pt,
  column-gutter: 41pt,
    [
- Borrow
- Rules of borrow
  - Only one owner per ressource
  - Only one mutable reference at a time
  - Or several immutables references
  - References must always be valid

  ],
  [
```rust
let patrick  = "scissor";
let sam = patrick;
```
    #line()
```rust
let patrick  = "scissor";
let sam = &patrick;
```
],
[
```rust
let mut patrick = String::from("scissor");
let sam = &patrick;
// cannot borrow `*sam` as mutable, as it is behind a `&` reference
sam.push_str("s")
```
  ],
    [

```rust
let mut patrick = String::from("scissor");
let sam = &mut patrick;
sam.push_str("s")
```
  ]
  )

]
#slide(title: "Lifetimes")[
- Each variable has a lifetime
- Compilator determine this to add instructions
- Possible to annotate lifetime

```rust
fn longest(x: &str, y: &str) -> &str {
  if x.len() > y.len() {
      x
  } else {
      y
  }
}
```

#text(fill: red, size: 20pt)[Error: missing lifetime specifier,
   this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`]
]

#slide(title: "Lifetimes")[
```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
  if x.len() > y.len() {
      x
  } else {
      y
  }
}
```

]

#slide(title: "Memory management")[
- No manual management of memory
- No garbage collector
- Use lifetime and ownership to ensure no memory leak

]

#slide(title: "Memory management")[
  What are the steps of the Rust compiler ?
]

#slide(title: "Memory management")[



#grid(
  columns: (2fr, 1fr),
    image("schemas/compiler-steps.png"),
    block(inset: 14pt)[
```rust
fn main() {
  let m = Box::new(2);
  let _a  = m;
}
```
]
)]



#slide(title: "Memory management")[
MIR Simplified

#place(
    top + right,
```rust






fn main() {
  let m = Box::new(2);
  let _a  = m;
}
```
  )
```rust
fn main() -> () {
    let mut _0: ();
    let _1: std::boxed::Box<i32>;
    scope 1 {
        debug m => _1;
        scope 2 {
            debug _a => _2;
        }
    }
    bb1: {
        _2 = move _1;
        drop(_2) -> [return: bb2, unwind continue];
    }
}
```

]

#slide(title: "Concurrency - thread-safe data structures")[

#grid(
  columns: (1fr, 1fr),
    block[
      In Java, OUPS...
```java
public class WebServer {
    ArrayList<User> users;

    @POST
    public Response createUser() {
        users.add(new User("John"));
        //...
    }
}
```],

block[
  #text(size: 19pt)[
  In Rust
```rust
struct Server {
    users: Rc<Vec<String>>,
}
impl Server {
    fn start(&self) {
        thread::spawn(move || {
            println!("{:?}", self.users);
        });
    }
}
```
]])

 #text(fill: red, size: 20pt)[Error: `Rc<Vec<String>>` cannot be shared between threads safely within `Server`,
   required for `&Server` to implement `std::marker::Send`]
]

#slide(title: "Concurrency - Mutex and Arc")[
#grid(
  columns: (1fr, 1fr),
    block[
      In C++, OUPS...
```cpp
class MegaCounter {
protected:
    int some_counter;

public:
    void save(int counter) {
        some_counter = counter;
    }
    int get() {
        return some_counter;
    }
};
```],

block[
  #text(size: 13pt)[
```rust
struct MegaCounter {
    some_counter: Mutex<i32>,
}
impl MegaCounter {
    fn new() -> Self { MegaCounter { some_counter: Mutex::new(0) } }

    fn increment(&self, add: i32) {
        // guard: MutexGuard<i32>
        let mut guard = self.some_counter.lock().unwrap();
        // mutable dereference to i32 via DerefMut trait
        *guard += add;
        // drop(guard);
    }

    fn get(&self) -> i32
    { *self.some_counter.lock().unwrap() }
}
fn main() {
    let counter = Arc::new(MegaCounter::new());
    for i in 0..10 {
        let arc = counter.clone();
        thread::spawn(move || {
            arc.increment(i);
        }); } }
```
]])
]


#slide(title: "Recap of the magic")[
  Borrow checker enforced rules
  - Only one mutable reference at a time
  - Or several immutables references
  - References must always be valid

  Why Rust ?
  - Combining no garbage collector and no manual memory management
  - Dynamic memory allocation with hidden free / drop
  - Minimal overhead at runtime
  - Whole package of memory safety issues removed
  - Data-races fixed, easier multi-threading
]

