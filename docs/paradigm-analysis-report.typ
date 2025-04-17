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

== Project needs
Before discussing this paradigm, let's briefly recall what DME (Delightful Markdown Experience) desktop app needs. As a Markdown previewer on steroids, we need to develop a pretty optimized program to bring a great browsing experience. To achieve maximum speed, we need to implement multi-threading to the maximum, making all IOs tasks in separated threads to avoid waiting on hardware when we could move forward with computation.

Searching for Markdown files on the disk, reading their content, indexing the headings, building a research index, is appropriate to sharing Markdown files across several threads to build this index as fast as possible on the first startup. In addition to building this index, we want to have a very fast rendering on the displayed Markdown document. Generating the highlighted code snippets can take a bit of time considering we'll use Tree-Sitter to have top-quality tokenization, that's another work to be distributed among several threads.

In addition, we want to avoid crashing the app as the whole UI will quit, creating a bad experience for the user, this could happen in case a strange Markdown file containing binary data was opened and the parser wasn't robust enough to support this unusual situation. It's not a like a CLI where if you get an error, you are used to run it again with other arguments, people are going to start it via the start menu and when it crashes, no logs will be immediately visible.

Finally, as the app is going to be open for hours, like a PDF previewer or a web browser, we cannot tolerate memory leaks as it would slowly but surely eat all the available RAM...

#pagebreak()

== Concurrent programming basics

Basic applications do not use the full power of modern processors when they only have one thread of execution. Having multiple CPU cores at disposition enable big performance gain by enabling parallel execution of calculation tasks or doing tasks repartition to separate UIs and background processing. When we start doing concurrent programming, managing several threads of execution come with major challenges.

To understand them, let's imagine a person responsible to create decorations for a Christmas tree.

A classic program would be, a person working first on cutting the paper for the décoration, then fold it and hang on the tree. Rince and repeat.
In concurrent programing, we would split the work with your family. Asking people 3 other people to help you with the 3/4 of the work. You work on your decoration, then hang on the tree. Simple, no ? Sadly, you only have one scissor, and only one of you can hang decoration on tree at a time.
In this metaphor, the scissor would be a shared ressource, and hanging decoration on the tree is a critical section. How you gonna manage this scissor without cutting the fingers of your mum when everyone want to use it ? We need protection mecanism to make sure the scissor is used safely and the usage is interupted by someone else as it could do damages on the tree or the fingers...

The root issue is that we don't control the order of execution, as the OS scheduler is the master in this situation. Standard solutions to control access to shared memory are mutex and semaphors but they can also be misused.

== Memory allocation basics

We generally represent the virtual memory of a process, with this kind of diagram. At left, the different regions dedicated to store the loaded code, the static variables and constants in BSS and Data, and finally the stack and the heap.

#image("schemas/empty.png")

Give this simplified piece of C code, we find numerous allocations in several places.
```c
#define SIZE 5

void print(int *toshow) {
    printf("%d", *toshow);
}

int main(void) {
    int a = 23;
    char *msg = "salut";
    char *ptr = malloc(sizeof(int) * SIZE);
    print(&a);
    free(ptr);
    return 0;
}
```

If we paused the program on `free`, the memory content would look like this. `a`, `msg`, `ptr` and `toshow` are local variables stored on the task.
We have 3 pointers:
- `msg` towards read-only data section pointing on `salut\0`
- `ptr` the dynamically allocated zone on the heap via `malloc`
- `toshow` pointing to the other variable `a` on the stack as we gave its address when calling the function

#image("schemas/filled.png")

== Why not just C++ or Java ?
#quote([You want performance for a desktop app, that's would be easy to build a C++ desktop app with Qt no ?])

*C++ would be a good option is terms of performance and object oriented paradigms* to manage and index the Markdown files. But there is a big issue regarding to concurrency. As we learned in the PCO course (Programmation Concurrente), we can spend hours reading small chunks of code managing mutexes and semaphors to make sure it is *correct* in terms of safety. We spent a lot of time checking and reviewing our own code and still failing to get everything right, sometimes with complicated deadlock hard to detect at first sight. 

The problem here is that the G++ cannot verify we are doing things correctly, as long as types are handled correctly, it can compile and the developer might detect nefarious bugs only in production. It's so easy to forget to protect a shared state, or associate a mutex in your head with 2 variables and forget a third one you just added.

For critical section where speed is really key, when using low level functions from C, we regularly take the risk of forgetting to freedeeeee heap allocated memory, leading to memory leaks.

#quote([You want to avoid memory safety issues ? Stop managing memory yourself and use Java !])

*Java would be good in terms of safety*, as all memory bugs almost disappear as we don't manage memory ourself and we cannot access raw pointers. As the JVM can generate exceptions, they can be catched to avoid a global crash, we can avoid most memory issues. This magic is coming from the garbage collector, regularly checking if our program has lost access so heap allocated data, to clean it itself. This is an annoying overhead that will not prevent us from reach maximum speed and avoid using extra RAMs.

Using Java also comes at a cost of executing our program in a virtual machine, instead of running our code directly on the CPU. The cold-start is known to be slow compared to native program. Finally, the concurrency issues do not go away, that's totally possible to use a `ArrayList` which is not thread-safe, instead of a safe equivalent (like `CopyOnWriteArrayList`).

== Why memory safety is a big deal ?
In a #link("https://github.com/microsoft/MSRC-Security-Research/blob/master/presentations/2019_02_BlueHatIL/2019_01%20-%20BlueHatIL%20-%20Trends%2C%20challenge%2C%20and%20shifts%20in%20software%20vulnerability%20mitigation.pdf")[Microsoft presentation from 2019], we find that #quote("~70% of the vulnerabilities addressed through a security update each year continue to be memory safety issues"). The Chromium projects #link("https://www.chromium.org/Home/chromium-security/memory-safety/")[also reports] that #quote("Around 70% of our high severity security bugs are memory unsafety problems (that is, mistakes with C/C++ pointers). Half of those are use-after-free bugs.").

To only cite a few, memory issues are use-after-free, buffer overflow, memory leaks, data race, ... They can causes big security issues as seen above, and cause app crashes, segmentation faults or data corruption.

== Performance + Memory safety: the best of both world
Rust is a strongly typed and compiled language, its strongest selling point is being the first language bringing the combination of speed and memory safety at the same time. It was designed for systems programming (browsers, kernels, ...) but now has reached almost all programming areas, even web frontend via webassembly.

== Why it is possible to get both ?
It doesn't use a garbage collector and doesn't ask the programmer to manually manage the memory. But how it is even possible ? How the program knows when to free heap allocated memory ?

The rust compiler `rustc` implement a new paradigm, including the notion of ownership and lifetimes, checked by a part of the compiler called the *borrow-checker*. Instead of associating only a type and a variable to a resource, like most modern languages, it also tracks who has the ownership of this resource and how long the resource must exist. When the variable is the owner of a resource, the resource will be deallocated when the variable go out of scope.

the borrow checker

=== The concept of ownership

Now, we know what is an shared ressource. While the usage of a mutex it can be a bit heavy.
In the end you could simple borrow the scissor then give it back when your finished.

Ownership is a programing concept introduced //TODO details
Each variable has an owner. This owner is responsible for the variable its lifetime and the memory management.
This would mean we can have a variable, scissor that we can share.

There is mupltple way to share a variable in Rust. The simple way is to move it.

```rust
let patrick  = "scissor";
let sam = patrick;
```
A simple assigment is enough to generate a move. Moving means changing the owner. The scissor are now under the responsibity of owner_2. Because it was moved, owner_1 cannot use it again.
Memory wise, owner_1 was responsible for the block of memory containtg the scissor.
Instead of moving it, we can instead lent it. In that case:

```rust
let patrick  = "scissor";
let sam = &patrick;
```

Here owner_2 asked for a reference, which imply a borrow instead of a move.
Memory-wise it would mean that the owner_2 has an access to the memory block containg "scissor"

This borrow can be mutable or not, meanig owner_2 is allowed to modify it or not.
```rust
let mut patrick = String::from("scissor");
let sam = &patrick;
// cannot borrow `*sam` as mutable, as it is behind a `&` reference
sam.push_str("s")
```

```rust
let mut patrick = String::from("scissor");
let sam = &mut patrick;
sam.push_str("s")
```

Now, we know how borrow works. Remember that the owner is responsible for the memory, meaning also dropping (freeing) it.

=== The concept of lifetime

To decorate our Chrismas tree, we decided to use candles. While a it is a really nice decoration, a candle end up burning out completly after a while.
It reached it end of its lifetime. Afterwards, we would need to clean base, and afterwards reuse it. Program do the same, they allocated memory to a variable, like our base for our candle,
then when this variable reached it end of its lifetime, the memory is freed so that the program can use it for something else, like reusing our base.

Most program use the scope to dertermine when a variable reached its end of its lifetime. Sometimes, a simple scope is not enough, so we need to extend the default lifetime of the variable.
As a programmer you know when your variable is not needed any longer, like how you know how long the candle will last. Because it's you who bought it, and your are its owner.

The goal to define the lifetime is to know when the candle is burnt out or the variable is unused. That way, the program can use the memory again. 
Instead of cleaning ourself, we let the program do it, but we indicate to him when he can cleanup if it is unclear for him

=== Why we don't need a garbage collector nor manual memory management ?

Compilator first define the owner of each variable. That way, the owner will be able to drop the variable when it reached the end of its lifetime, essentially freeing the memory allocated to for the variable. The owner can change via `move` but at the end we will only have one.

Behind the scene, when we allocate memory on the heap, the compilator will also determine when this memory will freed, adding in the source code the drop instruction when the associated pointer lifetime ends. While it is not directly visible, displaying the code generated by the compilator allow us to see hidden instructions.

This piece of code just stores the value `2` on the heap (Box is essentially a forced dynamic allocation), and then move the ownership of this pointer to another variable `_a`, making usage of `m` afterwards unauthorized.
```rust
fn main() {
    let m = Box::new(2);
    let _a  = m;
}
```

If we look at the generate code in intermediate step (MIR or Mid-level Intermediate Representation), by running `rustc -Z unpretty="mir" src/main.rs` with the nightly compiler, we get this:
```rust
fn main() -> () {
    let mut _0: ();
    let _1: std::boxed::Box<i32>;
    scope 1 {
        debug m => _1;
        let _2: std::boxed::Box<i32>;
        scope 2 {
            debug _a => _2;
        }
    }

    bb0: {
        _1 = Box::<i32>::new(const 2_i32) -> [return: bb1, unwind continue];
    }

    bb1: {
        _2 = move _1;
        drop(_2) -> [return: bb2, unwind continue];
    }

    bb2: {
        return;
    }
}
```
We see the `m` variable used as `_1` and `_a` variable used as `_2`. We can see in `bb1` makes the `move` explicit, then there is an added instruction drop, calling the destructor on `_2` (`_a`) as it's the new owner and there is thus only one drop!

=== The cost of the borrow-checker

The numerous benefits we get with the borrow-checker also come at a cost:
// actually no particular cost about time, this seems to be the LLVM and linking taking most of the time, not the borrow-checker...
- There is *learning curve steeper than other languages*, this is commonly referred as "fighting the borrow-checker", if you never worked with memory management before it will take time to get into it and start thinking about lifetimes and memory accesses yourself.
#figure(
  image("imgs/borrow-checker-meme.png", width: 48%),
  caption: [Meme from #link("https://www.reddit.com/r/rustjerk/comments/i4v5qa/borrow_checker/")[Reddit]],
)
- Some approaches like building graph data structures inherently have cross references, as *the borrow-checker is conservative* it prefers to reject correct programs instead of accepting wrong code, if one the rule is not respected it will not compile. #linebreak() There is an escape hatch with the `unsafe` keyword is that access some unsafe operations that need careful consideration, but it doesn't disable the other rest of the compiler checks. We can use `unsafe` blocks ourself or use wrapper types of the standard library that maintain `unsafe` sections for us. As the hardware is inherently unsafe, if when we want to write at a precise memory address, we need to manage some memory ourself. There is obviously the possibility to do errors in this unsafe code and create memory issues, but the surface area to check is far smaller that the entire codebase in a C codebase.
- *Rust is not the fastest way to build prototypes*, as it forces us to write safe code, before creating creating a binary. This issue can be partially mitigated with the "Rust easy mode", where you can clone everything with `.clone()` instead of using references, where you call `.unwrap()` on all `Result` or `Option` just to make it work: "it should not be any errors, and just crash if it's not the case".

=== Fearless concurrency

compilateur va utiliser les lifetimes pour savoir quand ya plus de ref le mutex et pour savoir quand libérer la mémoire.

auto unlock au drop du mutex

== Specifications for DME
Here the specifications of features we'll develop along the following weeks. We will work with [Tree-Sitter](https://tree-sitter.github.io/tree-sitter/) and [Tauri](https://tauri.app/).

=== Functional goals
*Preview*
+ We can preview any Markdown file with DME, by double-clicking on any `.md` file or running `dme test.md` in a GUI desktop app. Images and tables are supported.
+ The preview of code is done via Tree-Sitter: the syntaxes for C, C++, Java, Rust, Bash are built into DME (other languages are not supported for the start)
+ When the loaded file is changed on disk, the preview must refresh itself

*PDF export*
+ It's possible to export a PDF file with the same look as the preview via `dme export test.md`, including highlighted code with Tree-Sitter
+ It's possible to export several files at the same time just by giving more files like this `dme export test.md resume.md sample.md`

*Research*
+ It's possible to filter the list of Markdown with simple regexes in a configuration file in TOML, to avoid indexing hundreds of files unnecessary (i.e. when we cloned docs repository)
+ There is a way to quickly search among all Markdown files present on the disk, the search is matching the path fuzzily or matching terms inside headings
+ Finding a heading matching some keywords and choosing it will open the file as the current preview and jump to the matched heading
+ The research results are reloaded on every keypress, unlike a search engine
+ All documents under the user home directory can be found, except those present in folder starting with a dot (ignored `~/.config` i.e.)

=== Non-functional goals
All time measures must be made on Samuel's machine with a 12 cores processor and 16GB of RAM, with at least 5GB of unused RAM.

*Preview*
+ The preview of a ~10 pages document, with 50 pieces of code in different languages, must load under *300ms*
+ The refresh duration between when the file is saved and when the preview is updated, must be under *500ms*

*PDF export*
+ The PDF export of multiple files should be handled in parallel
+ The PDF export of a 10 pages document must take under 5s at maximum
+ The PDF export of 5 documents should not take more than 10s

*Research*
+ Building the full index of all headings of MDN content (Mozilla Developers Network documentation) should take less than *20s* (#link("https://github.com/mdn/content")[MDN GitHub repository], containing ~13000 `.md` files with ~95000 headings)
+ Searching for "Array constructor with a single parameter" (found on #link("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/Array#array_constructor_with_a_single_parameter")[this page]) by copy-paste must take less than 500ms to find the page on `Array() constructor`, showing the section `Array constructor with a single parameter`.
+ The partial search of "Array constructor single" should also list in the result the same section mentionned in the previous point

=== Applying the paradigm on DME
Keeping in the safe Rust subset (not using any `unsafe`), we'll be "forced" to follow of the ownership and lifetimes principles enforced by the borrow-checker. We will implement the 3 bigs features concurrently, to maximize the speed of each part.

== Sources
Our work is mainly based on our experience, reading several articles, documentations and watching videos. Here is what were the most useful to us in our research
- The Rust book - https://doc.rust-lang.org/book/ - mostly chapters 4 + 10.3 + 14 + 15
- The friendly and contextual error messages, once we learned the basic vocabulary, they really help to understand why the borrow-checker is not happy
- Rust book experiment - https://rust-book.cs.brown.edu/ - mostly chapter 4 with better explanations and visualisations of memory, ownership and lifetimes
- #link("https://www.technologyreview.com/2023/02/14/1067869/rust-worlds-fastest-growing-programming-language/")[How Rust went from a side project to the world’s most-loved programming language]
- #link("https://www.youtube.com/watch?v=HG1fppexRMA")[The Rust Borrow Checker - A Deep Dive - Nell Shamrell-Harrington, Microsoft] - Conference on Youtube
- #link("https://www.youtube.com/watch?v=Ju7v6vgfEt8")[How the Rust Compiler Works, a Deep Dive - RareSkills conf]
- #link("https://github.com/microsoft/MSRC-Security-Research/blob/master/presentations/2019_02_BlueHatIL/2019_01%20-%20BlueHatIL%20-%20Trends%2C%20challenge%2C%20and%20shifts%20in%20software%20vulnerability%20mitigation.pdf")[Microsoft presentation from 2019 on memory safety CVE]
- #link("https://www.chromium.org/Home/chromium-security/memory-safety/")[The Chromium Project - safety issues measures]

