use once_cell::sync::Lazy;
use std::collections::HashMap;

/// This is shown by the UI as proposed default links from the tree-sitter and tree-sitter-grammars Github organisations
pub static PROPOSED_GRAMMAR_SOURCES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        (
            "yaml",
            "https://github.com/tree-sitter-grammars/tree-sitter-yaml",
        ),
        (
            "lua",
            "https://github.com/tree-sitter-grammars/tree-sitter-lua",
        ),
        (
            "make",
            "https://github.com/tree-sitter-grammars/tree-sitter-make",
        ),
        (
            "toml",
            "https://github.com/tree-sitter-grammars/tree-sitter-toml",
        ),
        (
            "vue",
            "https://github.com/tree-sitter-grammars/tree-sitter-vue",
        ),
        (
            "csv",
            "https://github.com/tree-sitter-grammars/tree-sitter-csv",
        ),
        (
            "xml",
            "https://github.com/tree-sitter-grammars/tree-sitter-xml",
        ),
        ("cpp", "https://github.com/tree-sitter/tree-sitter-cpp"),
        ("php", "https://github.com/tree-sitter/tree-sitter-php"),
        ("rust", "https://github.com/tree-sitter/tree-sitter-rust"),
        ("scala", "https://github.com/tree-sitter/tree-sitter-scala"),
        ("css", "https://github.com/tree-sitter/tree-sitter-css"),
        ("regex", "https://github.com/tree-sitter/tree-sitter-regex"),
        ("html", "https://github.com/tree-sitter/tree-sitter-html"),
        ("java", "https://github.com/tree-sitter/tree-sitter-java"),
        ("bash", "https://github.com/tree-sitter/tree-sitter-bash"),
        (
            "typescript",
            "https://github.com/tree-sitter/tree-sitter-typescript",
        ),
        ("json", "https://github.com/tree-sitter/tree-sitter-json"),
        ("go", "https://github.com/tree-sitter/tree-sitter-go"),
        (
            "haskell",
            "https://github.com/tree-sitter/tree-sitter-haskell",
        ),
        ("c", "https://github.com/tree-sitter/tree-sitter-c"),
        (
            "javascript",
            "https://github.com/tree-sitter/tree-sitter-javascript",
        ),
    ])
});
