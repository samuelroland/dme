// Source: This comes from the unmaintained tree-painter crate released under MIT license
// and it has been is adapted to our situation
// https://github.com/matze/tree-painter
//
use crate::theming::{error::Error, theme};
use std::collections::HashMap;
use std::fmt::Write;

pub(crate) const HIGHLIGHT_NAMES: [&str; 27] = [
    "attribute",
    "comment",
    "constant",
    "constant.builtin",
    "constructor",
    "escape",
    "function",
    "function.builtin",
    "function.method",
    "function.macro",
    "include",
    "keyword",
    "label",
    "namespace",
    "number",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "repeat",
    "string",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

/// HTML syntax highlighting renderer.
pub struct Renderer {
    theme: theme::Theme,
    css_classes: HashMap<usize, String>,
}

impl Renderer {
    /// Create a new renderer based on `theme`.
    pub fn new(theme: theme::Theme) -> Self {
        let mut css_classes = HashMap::default();

        for index in theme.style_map.keys() {
            css_classes.insert(
                *index,
                format!(r#"class="tsc-{}""#, HIGHLIGHT_NAMES[*index]),
            );
        }

        Self { theme, css_classes }
    }

    /// Generate CSS block to be included in the `<style></style>` block or in an external CSS file.
    pub fn css(&self) -> String {
        let mut css = String::new();

        let _ = writeln!(
            css,
            ":root {{ --tsc-main-fg-color: {}; --tsc-main-bg-color: {}; }}",
            self.theme.foreground.color, self.theme.background.color
        );

        for (index, style) in &self.theme.style_map {
            let _ = write!(
                css,
                ".tsc-{} {{ color: {};",
                HIGHLIGHT_NAMES[*index], style.color
            );

            if style.is_bold {
                css.push_str("font-weight: bold;");
            }

            if style.is_italic {
                css.push_str("font-style: italic;");
            }

            css.push_str("}\n");
        }

        css.push_str(".tsc-line { word-wrap: normal; white-space: pre; }\n");
        css
    }
}
