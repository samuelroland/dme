#set text(font: "Cantarell")

#set page(
  numbering: "1",
  footer: align(
    center, 
    context(counter(page).display())
  )
)

#align(center)[
#text(size: 20pt)[= Ownership and lifetimes]
How Rust's unique feature will help us develop a safe and fast desktop app
#image("logo/logo.svg", height: 4em)
]

#outline() // TOC

== Needs
Before discussing this paradigm, let's briefly recall what DME (Delightful Markdown Experience) desktop app needs. We need to build several features that would greatly increase the experience if they are very optimized to be the fastest. To achieve maximum speed, we need to multi-threading to the maximum, making all IOs tasks in separated threads to avoid waiting on hardware when we could move forward with computation.

Searching for Markdown files on the disk, reading their content, indexing the full text, building a research index, is appropriate to sharing Markdown files across several threads to build this index as fast as possible on the first startup. In addition to building this index, we want to have a very fast rendering on the displayed Markdown document. Generating the highlighted code snippets can take a bit of time considering we'll use Tree-Sitter to have top-quality tokenization, that's another work to be distributed among several threads.

In addition, we want to avoid crashing the app as the whole UI will quit, creating a bad experience for the user, this could happen in case a strange Markdown file containing binary data was opened and the parser wasn't robust enough to support this unusual situation. It's not a like a CLI where if you get an error, you are used to run it again with other arguments, people are going to start it via the start menu and when it crashes, no logs will be immediately visible.

Finally, as the app is going to be open for hours, like a PDF previewer or a web browser, we cannot tolerate memory leaks as it would slowly but surely eat all the available RAM...

== Why not just C++ or Java ?
#quote([You want performance for a desktop app, that's would be easy to build a C++ desktop app with Qt no ?])

*C++ would be a good option is terms of performance and object oriented paradigms* to manage and index the Markdown files. But there is a big issue regarding to concurrency. As we learned in the PCO course (Programmation Concurrente), we can spend hours reading small chunks of code managing mutexes and semaphors to make sure it is *correct* in terms of safety. We spent a lot of time checking and reviewing our own code and still failing to get everything right, sometimes with complicated deadlock hard to detect at first sight. 

The problem here is that the G++ cannot verify we are doing things correctly, as long as types are handled correctly, it can compile and the developer might detect nefarious bugs only in production. It's so easy to forget to protect a shared state, or associate a mutex in your head with 2 variables and forget a third one you just added.

For critical section where speed is really key, when using low level functions from C, we regularly take the risk of forgetting to free heap allocated memory, leading to memory leaks.

#quote([You want to avoid memory safety issues ? Stop managing memory yourself and use Java !])

*Java would be good in terms of safety*, as all memory bugs almost disappear as we don't manage memory ourself and we cannot access raw pointers. As the JVM can generate exceptions, they can be catched to avoid a global crash, we can avoid most memory issues. This magic is coming from the garbage collector, regularly checking if our program has lost access so heap allocated data, to clean it itself. This is an annoying overhead that will not prevent us from reach maximum speed and avoid using extra RAMs.

Using Java also comes at a cost of executing our program in a virtual machine, instead of running our code directly on the CPU. Finally, the concurrency issues do not go away, that's totally possible to use a `ArrayList` which is not thread-safe, instead of a safe equivalent (like `CopyOnWriteArrayList`).

== Memory allocation basics

Give this simplified piece of C code, we find numerous allocation.
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
    return 0;
}
```

TODO explain quickly and explain why memory management is hard.
#image("schemas/empty.png")

#image("schemas/filled.png")

== the basics of memory management

== Why memory safety is a big deal ?
In a #link("https://msrc.microsoft.com/blog/2019/07/a-proactive-approach-to-more-secure-code/")[Microsoft presentation from 2019], we find that #quote("~70% of the vulnerabilities addressed through a security update each year continue to be memory safety issues"). The Chromium projects #link("https://www.chromium.org/Home/chromium-security/memory-safety/")[also reports] that #quote("Around 70% of our high severity security bugs are memory unsafety problems (that is, mistakes with C/C++ pointers). Half of those are use-after-free bugs.")

Memory related bugs generates 

== The best of both world
Rust is a strongly typed and compiled language, it has a strong selling point of being the first language bringing the combination of speed and memory safety at the same time. It was designed for systems programming (browsers, kernels, ...) but now has reached almost all programming areas, even web frontend via webassembly.

== How it is possible to get both ?
It doesn't use a garbage collector and doesn't ask the programmer to manually manage the memory. But how it is even possible ? How the program knows when to free heap allocated memory ?

The rust compiler `rustc` implement a new paradigm, including the notion of ownership and lifetimes, checked by a part of the compiler called the *borrow-checker*. Instead of associating only a type and a variable to a resource, like most modern languages, it also tracks who has the ownership of this resource and how long the resource must exist. When the variable is the owner of a resource, the resource will be deallocated when the 

the borrow checker

==== move + clone + ref + ref mut

==== lifetime
escabau
bougie fin du durée de vie, bougie consummée -> nettoyage du socle, réutilisation du socle pour une bougie.
move de socle move responsabilité de nettoyer

==== memory management
comment on fait pour ne pas avoir de garbage collector

==== how it is magic

==== le cout du borrow checker
cout de dev
courbe apprentissage
impossible à prouver que c'est safe même si c'est safe, unsafe Rust

==== gestion état partagé et communication inter threads

== cahier des charges de DME

== Introduction to concurrent programming

Fist of all, we need to understand what is conccurent programing and what are the main challenge of it.
Conccurent programing is about splitting tasks and working on them in parrallel,

To understand this, we will compare it to a person responsible to create decoration for a Christmas tree.

A classic program would be, a person working first on cutting the paper for the décoration, then fold it and hang on the tree. Rince and repeat.
In conccurent programing, we would split the work. For example split it in four, so asking people 3 other people to help you. 
You each work on your decoration, then hang on the tree. SImple, no ?

Sadly, you only have one scissor, and only one of you can hang decoration on tree at a time.
The scissor would be a shared ressource, and hanging decoration a tree a critical section.

Conccurent program always tried to address those issue. Mupltiple people hanging decoration on the tree could be dangerous, while there is only one scissor so the other have to wait on it.

A simple way to resolve this issue is using what we call a mutex. Mutex is like a lock on something, allowing only one person at a time working on it.

== The concept of ownership

Now, we know what is an shared ressource. While the usage of a mutex it can be a bit heavy.
In the end you could simple borrow the scissor then give it back when your finished.

Ownership is a programing concept introduced //TODO details
Each variable has an owner. This owner is responsible for the variable its lifetime and the memory management.
This would mean we can have a variable, scissor that we can share. 

There is mupltple way to share a variable in Rust. The simple way is to move it. 

```Rust
let owner_1  = "scissor";
let owner_2 = owner;
```
A simple assigment is enough to generate a move. Moving means changing the owner. The scissor are now under the responsibity of owner_2. Because it was moved, owner_1 cannot use it again. 
Memory wise, owner_1 was responsible for the block of memory containtg the scissor.
Instead of moving it, we can instead lent it. In that case:

```Rust
let owner_1  = "scissor";
let owner_2 = &owner;
```

Here owner_2 asked for a reference, which imply a borrow instead of a move. 
Memory-wise it would mean that the owner_2 has an access to the memory block containg "scissor"

This borrow can be mutable or not, meanig owner_2 is allowed to modify it or not.
```Rust
let mut owner_1  = "scissor";
let owner_2 = &owner;
//Is allowed because we said it's a mutable borrow
owner_2.append("s")
```

```Rust
let mut owner_1  = "scissor";
let owner_2 = mut & owner;
//Raise a compilation error because it is not a mutable borrow
owner_2.append("s")
```

Now, we know how borrow works. Remember that the owner is responsible for the memory, meaning also dropping (freeing) it.
== The concept of lifetime

Now 
== Memory management
//TODO move this    
I have a dream. A programming language where I don't have to manage memory. Most programmers out there knows that there is an easy solution. The garbage collector.
Sadly, garbage collector presents quiet a lot of issues. First of all, garbarge collecting is a complex action. You need to know when there is no more piece of code that is using a block of memory. 
Because of this complexity, garbage collecting is a heavy process, unsuitable for light appliction or appliction running in an environnement with low memory availabe.

Now then, as I said, I don't want to manually manage memory. It is prone to error, memory leak and responsible for a lot of break or insecure code. I want to avoid this risk.

Luckily for me, there is a solution. But to explains how it works, we first need to introduce a few concept.
Lifetime and ownership.

= DME, Delightful Markdown Experience
//TODO Cahier des charges
