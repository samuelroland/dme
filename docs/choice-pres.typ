// Presentation about the paradigm choice and language

// Sous la forme d'une présentation de maximum 10 minutes, vous présentez à la classe votre choix :
//     Quel paradigme, ainsi que le langage associé ;
//     Pourquoi (particularité du paradigme et/ou du langage) ;
//     Comment prévoyez-vous de l'utiliser (courte description du projet qui mettra en avant les particularités du paradigme et/ou langage).

#import "@preview/typslides:1.2.3": * // https://github.com/manjavacas/typslides
#import "@preview/tablex:0.0.8": tablex

// Project configuration
#show: typslides.with(
  ratio: "16-9",
  theme: "yelly",
)

#text(font: "Cantarell", [


#blank-slide[
  #align(center, [
  #yelly[#text(weight: "bold", size: 2.2em, fill: gradient.linear(rgb("#fc1100"),rgb("#ffb000")))[Delightful Markdown Experience ?]]
  = That's possible !
  ])
]


#slide(title: "Meta experience")[
== Current experience
 - Preview not always pleasant
 - Code highlighting too much basic
 - PDF export hard and broken
 - Single file preview

== Dream experience
- Jumping easily through any Markdown file on disk
- Full text search on Markdown content
- Fast preview load and refresh, even for very big documents
- Full code highlighting with Tree-Sitter
]

#slide(title: "Challenges")[
- Time of Markdown files research
- Time of indexing Markdown content
- Speed of full text search
- Speed of HTML & Code preview generation
- Speed of PDF generation

]
#blank-slide[
  #align(center, [
  #yelly[#text(weight: "bold", size: 2.2em, fill: gradient.linear(rgb("#fc1100"),rgb("#ffb000")))[Ownership and lifetimes]]
  = Applied to concurrent programming
  ])
]
#slide(title: "Classic approach")[
== Memory safety in C++

```rust
void task(int *counter) {
    while (*counter < 10000000)
        (*counter)++;
}

const int NB_THREADS = 30;

int main(void) {
    int counter = 0;
    PcoThread *threads[NB_THREADS];
    for (int i = 0; i < NB_THREADS; i++)
        threads[i] = new PcoThread(task, &counter);

    for (int i = 0; i < NB_THREADS; i++)
        threads[i]->join();

    cout << "counter " << counter << endl;
}
```

```sh
> ./build/exo
counter 10000000
```

```sh
> # running 10 times
Results
 6 times counter 10000000
 2 times counter 10000001
 2 times counter 10000002
```
]

#slide(title: "Another approach")[
== Memory safety in Rust
```rust
fn task(counter: &mut u32) {
    while *counter < 100000 {
        *counter += 1;
    }
}

const NB_THREADS: u32 = 30;

fn main() {
    let mut counter: u32 = 0;
    let mut handles = Vec::new();
    for _ in 1..NB_THREADS {
        handles.push(thread::spawn(|| task(&mut counter)));
    }
    for handle in handles {
        handle.join();
    }
    println!("counter {counter}");
}
```

```rust
error[E0373]: closure may outlive the current function, but it borrows `counter`, which is owned by the current function
  --> src/main.rs:14:36
   |
14 |         handles.push(thread::spawn(|| task(&mut counter)));
   |                                    ^^           ------- `counter` is borrowed here
   |                                    |
   |                                    may outlive borrowed value `counter`
   |   
```


```rust
error[E0499]: cannot borrow `counter` as mutable more than once at a time
  --> src/main.rs:14:36
   |
14 |         handles.push(thread::spawn(|| task(&mut counter)));
   |                      --------------^^--------------------
   |                      |             |            |
   |                      |             |            borrows occur due to use of `counter` in closure
   |                      |             `counter` was mutably borrowed here in the previous iteration of the loop
   |                      argument requires that `counter` is borrowed for `'static`
```

```rust
error[E0502]: cannot borrow `counter` as immutable because it is also borrowed as mutable
  --> src/main.rs:19:23
   |
14 |         handles.push(thread::spawn(|| task(&mut counter)));
   |                      ------------------------------------
   |                      |             |            |
   |                      |             mutable borrow occurs here
...
19 |     println!("Counter {counter}");
   |                       ^^^^^^^^^ immutable borrow occurs here
```

]

#slide(title: "Why Rust ?")[
== Memory safety
- Concurrent access checked at compile time
- Strong typing system, smart types like ```Mutex```
- No garbage collector and no manual memory management

]

])

