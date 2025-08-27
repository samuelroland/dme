// Source: This comes from the unmaintained tree-painter crate released under MIT license
// and it has been is adapted to our situation
// https://github.com/matze/tree-painter
//
use crate::theming::theme;
use std::fmt::Write;

/// HTML syntax highlighting renderer.
pub struct Renderer<'a> {
    theme: &'a theme::Theme<'a>,
}

/// Only apply the generated css when we are inside <code> (inline and block)
/// Just to avoid CSS names conflicts with other
const CSS_SCOPE: &str = "code";

impl<'a> Renderer<'a> {
    /// Create a new renderer based on `theme`.
    pub fn new(theme: &'a theme::Theme) -> Self {
        Self { theme }
    }

    /// Generate CSS block to be included in the `<style></style>` block or in an external CSS file.
    /// The generated classes are based on all available highlighting names defined in the `theme`
    pub fn css(&self) -> String {
        let mut css = format!(
            "pre {{background-color:{};}}\n{} {{color:{};}}\n",
            self.theme.background.color, CSS_SCOPE, self.theme.foreground.color
        );

        // Just sort the vec in tests to allow regression tests on the output
        // This is changing the order of iteration otherwise
        let mut styles: Vec<(_, _)> = self.theme.style_map.iter().collect();
        styles.sort();
        for (index, style) in styles {
            let _ = write!(
                css,
                "{} .{}{{color:{};",
                CSS_SCOPE, self.theme.supported_highlight_names[*index], style.color
            );

            if style.is_bold {
                css.push_str("font-weight:bold;");
            }

            if style.is_italic {
                css.push_str("font-style:italic;");
            }

            css.push_str("}\n");
        }

        // TODO: remove that if that's actually useless
        // css.push_str(".tsc-line { word-wrap: normal; white-space: pre; }\n");
        css
    }
}

#[cfg(test)]
mod tests {
    use std::{env::current_dir, fs::read_to_string};

    use crate::theming::theme::Theme;

    use super::Renderer;

    #[test]
    fn test_can_render_css_for_catppuccin_latte_theme() {
        let content = read_to_string(
            current_dir()
                .unwrap()
                .join("src/theming/default/catppuccin_latte.toml"),
        )
        .unwrap();
        let theme = Theme::from_helix(&content, &["variable", "function", "markup.bold"]).unwrap();

        let renderer = Renderer::new(&theme);
        // Simple sorter by lines
        let sorter = |given: &str| -> String {
            let mut lines = given.lines().collect::<Vec<&str>>();
            lines.sort();
            lines.join("\n")
        };
        assert_eq!(sorter(&renderer.css()), sorter("code .function{color:#1e66f5;}\ncode .markup.bold{color:#d20f39;font-weight:bold;}\ncode .variable{color:#4c4f69;}\ncode {color:#4c4f69;}\npre {background-color:#eff1f5;}"));

        // TODO: we could try to minimize later the size of the generated CSS
        // I see that color in variable is from "text" var in TOML file so it's a duplicated from
        // the default value given in the first CSS default rule.
    }
}
