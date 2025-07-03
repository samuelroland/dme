/// Security testing, internally to allow access of pub(crate) functions
#[cfg(test)]
mod tests {
    use crate::{
        highlight_code,
        preview::{
            comrak::ComrakParser,
            preview::Previewable,
            tree_sitter_grammars::{
                get_test_grammar_repos, get_unique_local_tree_sitter_grammars_folder,
                TreeSitterGrammarsManager, TEST_GRAMMAR,
            },
            tree_sitter_highlight::TreeSitterHighlighter,
        },
    };
    use pretty_assertions::assert_eq;
    use tree_sitter_loader::Loader;
    #[test]
    fn test_highlight_code_cannot_inject_markdown_nor_html() {
        let snippet = r#"color: blue;
```
<h1>injected title !</h1>
```"#;
        let result1 = highlight_code("", snippet).unwrap();
        let result2 = highlight_code("unknown", snippet).unwrap();
        assert_eq!(
            result1.to_safe_html_string(), // the backticks are left as is, the html is escaped
            "color: blue;\n```\n&lt;h1&gt;injected title !&lt;/h1&gt;\n```"
        );
        assert_eq!(result1.to_safe_html_string(), result2.to_safe_html_string());
    }

    #[test]
    // Escaping is already handled by tree-sitter-highlight, just to make sure it continues to work on the long run
    // Quotes are not escaped as this is only text inside tags, not in attributes
    fn test_highlight_code_escapes_html_even_if_unrelated_to_lang() {
        let mut m = TreeSitterGrammarsManager::new_with_grammars_folder(
            get_unique_local_tree_sitter_grammars_folder(),
        )
        .unwrap();

        m.install(&get_test_grammar_repos()).unwrap();
        let snippet = "<div>This is an HTML page</div><SCRIPT>alert('yoo')</script>";
        let mut loader = Loader::new().unwrap();
        let h = TreeSitterHighlighter::new(&mut loader, TEST_GRAMMAR, &m).unwrap();
        assert_eq!(
            h.highlight(snippet).to_safe_html_string(),
            r#"&lt;div<span class="operator">&gt;</span><span class="tag">This</span> <span class="tag">is</span> <span class="tag">an</span> <span class="tag">HTML</span> <span class="tag">page</span>&lt;<span class="operator">/</span>div<span class="operator">&gt;</span>&lt;SCRIPT<span class="operator">&gt;</span><span class="tag">alert</span><span class="punctuation bracket">(</span><span class="string">'yoo'</span><span class="punctuation bracket">)</span>&lt;<span class="operator">/</span>script<span class="operator">&gt;</span>
"#
        );

        let snippet = "\" onload='alert(true)'><span \"";
        assert_eq!(
            h.highlight(snippet).to_safe_html_string(),
            r#"<span class="string">" onload='alert(true)'&gt;&lt;span "</span>
"#
        );
    }

    #[test]
    // We have to take care about that ourself !
    fn test_invalid_languages_are_html_escaped() {
        let parser = ComrakParser::new().unwrap();
        let snippet =
            "# Hello\n```zonk\n<div>This is an HTML page</div><SCRIPT>alert('yoo')</script>\n```";
        let html = parser.to_html(snippet);
        assert_eq!(
        html.to_safe_html_string(),
            "<h1>Hello</h1>\n<pre><code class=\"language-zonk\">&lt;div&gt;This is an HTML page&lt;/div&gt;&lt;SCRIPT&gt;alert('yoo')&lt;/script&gt;\n</code></pre>\n"
        );
    }

    #[test]
    fn test_html_in_document_is_not_escaped_but_cleaned() {
        let parser = ComrakParser::new().unwrap();
        let snippet = "# Hello\n<div>This is an HTML page</div><SCRIPT>alert('yoo')</script>\n";
        let html = parser.to_html(snippet);
        assert_eq!(
            html.to_safe_html_string(),
            "<h1>Hello</h1>\n<div>This is an HTML page</div>\n"
        );
    }
}
