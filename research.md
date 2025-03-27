# Research on Ownership and lifetimes paradigm

## Things to understand absolutely
- Basics of the stack and the heap, stack frames, pointers, dynamic allocation
- What's are the high level steps of the compiler,
    - where is the borrow checker
    - what are the different substeps in the borrowchecker
    - https://rustc-dev-guide.rust-lang.org/borrow_check.html
- Automatic and dynamic allocations models
    - `Box` for heap allocation, when this is useful
    - Trait `Drop`, `Clone`, `Copy`, `Sync`, `Send`
    - how does the compiler treat them
    - which conditions are imposed on which struct can implement them -> can any struct implement `Sync`
- Threads management
    - Start, stop, join
- Async and borrowchecker
    - How does work async code ?
    - `Future` trait
    - `Rc`, `Arc`, `Mutex` types, what they enable
- Lifetimes annotation like `'a`
    - how multiple lifetimes work together
    - what are the exact rules behind lifetimes verification
- Complex data types like graphs, trees
    - Why it doesn't work with simple types
    - How std types like `RefCell`, `Cell` works
    - Dynamic borrow checking verifications like in `RefCell`
- Unsafe Rust
    - What are the additionnal rules of what is authorized and why this is necessary sometimes


## Videos
The Rust Borrow Checker - A Deep Dive - Nell Shamrell-Harrington, Microsoft
https://www.youtube.com/watch?v=HG1fppexRMA
The Rust Borrow Checker: a Deep Dive - Nell Shamrell-Harrington
https://www.youtube.com/watch?v=JfEWmQAACN8
Really starts at `2:56`

TIL
- This is not linear but the simplification is that there are 5 steps of compilation: lexical analysis, parsing, semantic analysis, optimization, code generation. The borrow checker is the semantic analysis.
- The borrow checker is working the MIR the Mid level Intermediate Representation
- The most recommended guide is Rust Compiler Development Guide, at https://rustc-dev-guide.rust-lang.org/

33min
The Rust Borrow Checker: a Deep Dive - Nell Shamrell-Harrington
https://www.youtube.com/watch?v=JfEWmQAACN8

Rust for TypeScript devs : Borrow Checker - The Primeagen
https://www.youtube.com/watch?v=ZNFdkTIzdXM

Understand Rust's Borrow Checker in 5 Minutes
https://www.youtube.com/watch?v=Nuba5LNy5cY

How to fight Rust's borrow checker... and win. - Let's Get Rusty
https://www.youtube.com/watch?v=Pg07HQJ0tvI

How the Rust Compiler Works, a Deep Dive - RareSkills conf
https://www.youtube.com/watch?v=Ju7v6vgfEt8

TIL - useful between `00:07:30` - `01:29:00` - to listen in `2.5x`
- The steps: - Lexing - Parsing - Semantic analysis -> AST - HIR -> THIR (Typed High level Intermediate Representation) -> MIR borrow checker -> LLVM IR
- `rustc -Z help` on the nightly compiler provides a huge list of flags, some of them allow dumping internal representations.
    - `rustc -Z unpretty=mir src/main.rs` -> dump the MIR

AsRef/Borrow Traits, and the ?Sized Marker - Rust [Video Request]
https://www.youtube.com/watch?v=4YAmpHMl1Z0

04 Ownership & Borrowing | Rust Tutorials
https://www.youtube.com/watch?v=q2UnbA2dkc8

RustEdu Workshop 2022 - RustViz: Interactively Visualizing Ownership and Borrowing
https://www.youtube.com/watch?v=zCF8QVkc6IY

## Useful
Decrusting the serde crate
https://www.youtube.com/watch?v=BI_bHCGRgMY

## Channels
Ryan Levick
https://www.youtube.com/channel/UCpeX4D-ArTrsqvhLapAHprQ

https://www.youtube.com/watch?v=NQBVUjdkLAA

Jon Gjengset - Crust of Rust
https://www.youtube.com/c/JonGjengset

## Rustc dev guide
### The MIR (Mid-level IR)
https://rustc-dev-guide.rust-lang.org/mir/index.html
TO CONTINUE

### MIR borrow check
https://rustc-dev-guide.rust-lang.org/borrow_check.html

### Unsafety Checking
https://rustc-dev-guide.rust-lang.org/unsafety-checking.html

### Drop elaboration
https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html


### The RFC of the MIR introduced in 2015
https://rust-lang.github.io/rfcs/1211-mir.html 
- Main reason of introducing it:
    1. "The complexity of the compiler is increased because all passes must be written against the full Rust language, rather than being able to consider a reduced subset."
    1. "Reasoning about fine-grained control-flow in an AST is rather difficult. The right tool for this job is a control-flow graph (CFG)"

### The Rust Reference - Destructors
https://doc.rust-lang.org/reference/destructors.html

## Articles

Exploring Dataflow Analysis in the Rust Compiler 
https://aneksteind.github.io/posts/2023-06-12.html
