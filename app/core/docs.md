# High level overview of the library
This is coming to complet the inline triple slash comments to have a high level overview and understand design decisions.

## Preview

### Tree-Sitter grammars management
To avoid a big binary size and choosing which grammar to bundle in the final binary, we are taking a different approach. No grammar at all will be bundled and they will be downloaded, compiled and loaded at runtime. We provide a short list of default Git repositories on the Tree-Sitter Github organisation and the Tree-Sitter Grammars Github organisation. The user will be able to use other Git links as well.

It will provide a mecanism to install, update and remove any grammar. A grammar is just a git repository named `tree-sitter-<something>` where `<something>` is generally the language identifier. For example the YAML grammar is at https://github.com/tree-sitter-grammars/tree-sitter-yaml, the file `grammar.js` defines the grammar and the generated C parser has been commited under `src` folder.

These decisions require the user to have a C compiler installed and Git.

#### Where to store these grammars ?

Grammars will managed separately from already available grammars defined in Tree-Sitter `config.json`, to not interfere with an existing development environment. Installing syntaxes during development will be maybe supported later. We want to store it in a folder that is not specific to DME, because other software could reuse them as well (like Delibay or PLX).

**The default location of these grammars** is in th home data folder (for the [the XDG base directory specification](https://specifications.freedesktop.org/basedir-spec/latest/#basics) that's under `~/.local/share/tree-sitter-grammars` subfolder. Choosing this folder in a cross-plateform way is achieved via [`etcetera`](https://crates.io/crates/etcetera) crate.

The environment variable `TREE_SITTER_GRAMMARS_FOLDER` can be changed outside of the program to override the folder at the level at of the `ComrakParser`. It will use the default 

During tests, we redefine this entry to a unique subfolder like `target/tree-sitter-grammars/608257137`. It must be unique to avoid interference between tests.

In fact we don't even have a `config.json` on disk, we only use a configuration in memory where we push the folder

#### How we install these grammars
Given a Git link like `https://github.com/tree-sitter-grammars/tree-sitter-yaml`
- We extract the repos name from url -> `tree-sitter-yaml`
- We stop if there is already a folder with this name
- We run `git clone https://github.com/tree-sitter-grammars/tree-sitter-yaml` inside `~/.local/share/tree-sitter-grammars`
- Run compilation once via `Loader::load_language_at_path()` after having configured to force recompilation, it will create a `~/.local/share/tree-sitter-grammars/tree-sitter-yaml/yaml.so` shared library to be loaded later.

We can then update, remove or list these grammars based on the lang identifier. If want to remove the grammar for id `css`, it will search for a folder `tree-sitter-css` inside the grammars folder. The update runs `git pull` and trigger a recompilation.

#### How the highlight process work ?
1. We load the

            // Note: tree-sitter.json contains an array of `grammars` which could be more than one
            // grammar sometimes (typescript -> typescript, tsx and flow. xml -> xml and dtd)
            // For now, we only support the first entry.


        // Note: we making the supposition that the lang is in the folder name, for now
We don't support local queries and injections queries for now.



    /// Get a slice containing all of the highlight names used in the configuration.
    #[must_use]
    pub const fn names(&self) -> &[&str] {

doesnt support language injection



    /// Parse highlight names from queries files as we need to give a list of recognized
    /// names, we want to accept all of them
    /// highlight name is something like "type.builtin" "variable.local" "keyword" "constant"
    /// The whole list of supported names for Helix themes are here
    /// https://docs.helix-editor.com/themes.html#scopes
