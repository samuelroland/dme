# Research on Ownership and lifetimes paradigm

## Things to understand absolutely
- Basics of the stack and the heap, stack frames, pointers, dynamic allocation
- What's are the high level steps of the compiler,
    - where is the borrow checker
    - what are the different substeps in the borrowchecker
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

33min
The Rust Borrow Checker: a Deep Dive - Nell Shamrell-Harrington
https://www.youtube.com/watch?v=JfEWmQAACN8

Rust for TypeScript devs : Borrow Checker - The Primeagen
https://www.youtube.com/watch?v=ZNFdkTIzdXM

Understand Rust's Borrow Checker in 5 Minutes
https://www.youtube.com/watch?v=Nuba5LNy5cY

How to fight Rust's borrow checker... and win. - Let's Get Rusty
https://www.youtube.com/watch?v=Pg07HQJ0tvI
