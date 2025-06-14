# High level overview of the library
This is coming to complet the inline triple slash comments to have a high level overview and understand design decisions.

## Markdown parsing
We use `comrak` to parse Markdown. We integrate Tree-Sitter highlighting via the `SyntaxHighlighterAdapter` trait that allow us to call our `TreeSitterHighlighter` on each code blocks.

## Preview

### What's a Tree-Sitter grammar ?
A grammar is just a git repository named `tree-sitter-<something>` where `<something>` is generally the language identifier. For example the official CSS grammar is at `https://github.com/tree-sitter/tree-sitter-css`, the file `grammar.js` defines the grammar and the generated C parser has been commited under `src` folder.

The `tree-sitter.json` at repository's root looks like that. We have a list of grammars
```json
{
  "grammars": [
    {
      "name": "css",
      "camelcase": "CSS",
      "scope": "source.css",
      "path": ".",
      "file-types": [
        "css"
      ],
      "highlights": "queries/highlights.scm",
      "injection-regex": "^css$"
    }
  ],
  // ..
}
```

It contains a `grammar.js` which defines the rules to generate to tokenize source code.

```js
...
 rules: {
    stylesheet: $ => repeat($._top_level_item),

    // Statements
    import_statement: $ => seq(
      '@import',
      $._value,
      sep(',', $._query),
      ';',
    ),
...
```

This definition is used to generate a C parser, which is usually commited directly in the repository, we don't need to generate that in this library. This parser  under `src` contains a `parser.c`
```sh
src> tree
├── grammar.json
├── node-types.json
├── parser.c
├── scanner.c
└── tree_sitter
    ├── alloc.h
    ├── array.h
    └── parser.h
```

This parser need to be built into a shared library before we can load it dynamically via the `tree-sitter` crate which are Rust bindings to the C library.

Once a source code has been parsed into a tree of nodes representing each token, it still need to attributes highlight names. A highlight name is something like `type.builtin`, `variable.local`, `keyword` or `constant` which indicate the type of the token in a generic theming way. These names are not specific to a language, contrary to nodes types who are different for each language.

As we see in the `tree-sitter.json` configuration, each grammar can indicate one or more highlight query files like here `highlights: queries/highlights.scm`. Here is the small extract that shows some tokens containing literal char are associated with the `operator` highlight name. Then, the token of type `class_name` get attributed the `property` name.

```scm
"~" @operator
">" @operator
"+" @operator
"-" @operator
...

(class_name) @property
(id_name) @property
(namespace_name) @property
(property_name) @property
```

These names are then used to generate the class names in HTML like `<span class="operator">-</span>`. There is specification regarding the valid highlight names, it depends on the IDE and theme system. We use the themes from Helix, you can see the full list in [Helix Themes documentation](https://docs.helix-editor.com/themes.html#syntax-highlighting).

A Helix theme is a TOML file looks like that.

```toml
"constant" = "peach"
"constant.character" = "teal"
"constant.character.escape" = "pink"

"string" = "green"
"string.regexp" = "pink"
"string.special" = "blue"
"string.special.symbol" = "red"
...

[palette]
rosewater = "#dc8a78"
flamingo = "#dd7878"
pink = "#ea76cb"
mauve = "#8839ef"
```

Finally, we are using a theme to generate the CSS for each supported highlight names.
```css
code .function {
  color: #1e66f5;
}
code .markup.bold {
  color: #d20f39;
  font-weight: bold;
}
code .variable {
  color: #4c4f69;
}
/* General look */
code {
  color: #4c4f69;
}
pre {
  background-color: #eff1f5;
}
```

### Theming
Currently, we only have one default theme **Catpuccin Latte** for light usage. Later we will probably develop a themes manager.

### Tree-Sitter grammars management
To avoid a big binary size and choosing which grammar to bundle in the final binary, we are taking a different approach. No grammar at all will be bundled and they will be downloaded, compiled and loaded at runtime. We provide a short list of default Git repositories on the Tree-Sitter Github organisation and the Tree-Sitter Grammars Github organisation. The user will be able to use other Git links as well.

It will provide a mecanism to install, update and remove any grammar.
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
1. First the creation of `TreeSitterHighlighter` via `new()`
    1. We normalize the given lang id (so `js` become `javascript`)
    1. We search a grammar inside the grammars folder with name `tree-sitter-lang` where `lang` is the language id after normalisation)
    1. We load it via `Loader::load_language_at_path()` which returns a `Language`. After loading the first language configuaration for this grammar folder, we can run `LanguageConfiguration::highlight_config()` with the previous `language` and if it succeeded it has loaded internally all the queries files and parsed all the highlight names present in all queries files of the grammar. We get a `HighlightConfiguration` which allow us to get access via `HighlightConfiguration::names()` to the list of detected names later.
1. Then, each call to `highlight()` will do the job based on the `highlight_config` saved as attribute, we apply the highlight name as a CSS class with `.` replaced by a space. This is where we use `self.highlight_config.names()`.

#### Limits
Looking at syntaxes via folder name is not the best approach, that's a temporary simplification. The `tree-sitter-typescript` contains multiple grammars: `typescript`, `tsx` and `flow` and we only consider the first for now. A code highlighted as `tsx` will not receive any highlights for now.

We don't support local queries and injections queries for now. If a language contains another language, this integrated language will not be highlighted (this is called language injection).
