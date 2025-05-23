// Presentation about the paradigm choice and language

// Sous la forme d'une présentation de maximum 10 minutes, vous présentez à la classe votre choix :
//     Quel paradigme, ainsi que le langage associé ;
//     Pourquoi (particularité du paradigme et/ou du langage) ;
//     Comment prévoyez-vous de l'utiliser (courte description du projet qui mettra en avant les particularités du paradigme et/ou langage).

#import "@preview/typslides:1.2.3": * // https://github.com/manjavacas/typslides
#import "@preview/tablex:0.0.9": tablex

// Project configuration
#show: typslides.with(
  ratio: "16-9",
  theme: "yelly",
)
#set text(font: "Cantarell")

#blank-slide[
  #align(center, [
  #image("logo/logo.svg", height: 5em)
  #yelly[#text(weight: "bold", size: 2.2em, fill: black)[A delightful Markdown experience ?]]
  = That's possible !
  ])
]


#slide(title: "Markdown experience outside editing")[
== Current experience
- Preview not always pleasant
- Code highlighting too much basic
- PDF export very hard and broken
- Single file preview

== Dream experience
- Jumping easily through any Markdown file on disk
- Full text search on Markdown content
- Fast preview load and refresh, even for very big documents
- Full code highlighting with Tree-Sitter
- Easy export in PDF
]

#slide(title: "Challenges")[

#tablex(
  columns:2,
  stroke: none,
  inset: 0pt,
  align: horizon,
  row-gutter: 0em,
  column-gutter: 2em,
)[
== Speed
- Markdown files research
- Markdown content indexing 
- Full text search
- HTML & Code preview generation
- PDF generation
][
    #image("imgs/feris-car.png", height: 19em)
]

]
#blank-slide[
  #align(center, [
  #yelly[#text(weight: "bold", size: 2.2em, fill: gradient.linear(rgb("#fc1100"),rgb("#ffb000")))[Ownership and lifetimes]]
  = Applied to concurrent programming
  ])
]

#slide(title: "Memory safety in concurrency")[
== C++ vs Rust
#text(size: 16pt)[
#tablex(
  columns: 2,
  stroke: none,
  inset: 0pt,
  row-gutter: 0em,
  column-gutter: 2em,
)[
```cpp
void task(int *counter) {
    while (*counter < 10000000)
        (*counter)++;
}

int main(void) {
    int counter = 0;
    PcoThread *threads[30];
    for (int i = 0; i < 30; i++)
       threads[i] = new PcoThread(task, &counter);
    // [...] joining threads
    cout << "counter " << counter << endl;
}
```
][
```rust
fn task(counter: &mut u32) {
    while *counter < 10000000 { *counter += 1; }
}

fn main() {
    let mut counter = 0;
    let mut handles = Vec::new();
    for _ in 1..30 {
        handles.push(
            thread::spawn(|| task(&mut counter))
        );
    }
    // [...] joining threads
    println!("counter {counter}");
}
```
]]]

#slide(title: "Memory safety in concurrency")[
== Results
#text(size: 16pt)[
#tablex(
  columns:2,
  stroke: none,
  inset: 0pt,
  row-gutter: 0em,
  column-gutter: 2em,
)[
```sh
> # BUILD OK
> # running once
counter 10000000
```

```sh
> # running 10 times
Results
 6 times: counter 10000000
 2 times: counter 10000001
 2 times: counter 10000002
```
][
```rust error:```#text(weight: "bold")[ closure may outlive the current function, but it borrows `counter`, which is owned by the current function]```rust handles.push(thread::spawn(|| task(&mut counter)));
        `counter` is borrowed here
         may outlive borrowed value `counter`
```
```rust error:```#text(weight: "bold")[ cannot borrow `counter` as mutable more than once at a time]```rust handles.push(thread::spawn(|| task(&mut counter)));
            `counter` was mutably borrowed here in the previous
            iteration of the loop
```

```rust error:```#text(weight: "bold")[ cannot borrow `counter` as immutable because it is also borrowed as mutable]```rust handles.push(thread::spawn(|| task(&mut counter)));
                              mutable borrow occurs here
println!("Counter {counter}");
                  ^^^^^^^^^ immutable borrow occurs here
```

]
]
]

#slide(title: "Why Rust ?")[
== Memory safety and speed
- Concurrent access checked at compile time
- Strong typing system, smart types like `Mutex`
- No garbage collector and no manual memory management
]


