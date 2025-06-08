// Source: This comes from the unmaintained tree-painter crate released under MIT license
// and it has been is adapted to our situation
// https://github.com/matze/tree-painter
use crate::theming::error::Error;
use std::collections::HashMap;
use std::convert::From;
use toml::value::Table;
use toml::Value;

pub(crate) struct Style {
    pub color: String,
    pub is_bold: bool,
    pub is_italic: bool,
}

impl From<&String> for Style {
    fn from(color: &String) -> Self {
        Style {
            color: color.clone(),
            is_bold: false,
            is_italic: false,
        }
    }
}

/// A theme defining colors and modifiers to be used for syntax highlighting.
pub struct Theme {
    pub(crate) style_map: HashMap<usize, Style>,
    pub(crate) foreground: Style,
    pub(crate) background: Style,
    pub(crate) supported_highlight_names: Vec<String>,
}

impl Theme {
    /// Load theme from a Helix [compatible](https://docs.helix-editor.com/themes.html) theme
    /// description stored in `data`.
    ///
    /// # Errors
    ///
    /// If the theme cannot be parsed either because it is not a TOML file or does not adhere to
    /// the Helix syntax expectations, this function returns an [`Error`].
    ///
    /// Make sure to include all possible highlighting names inside this theme
    /// for all the languages you are highlighting ! We are giving an external list to limit the
    /// amount of keys to generate CSS for
    /// Note: we might change that in the future to generate the whole CSS definitions for any
    /// highlighting name used in the theme itself
    pub fn from_helix(data: &str, supported_highlight_names: Vec<String>) -> Result<Self, Error> {
        let root = match data.parse::<toml::Value>()? {
            Value::Table(table) => table,
            _ => return Err(Error::InvalidTheme),
        };

        let palette = root.get("palette").ok_or(Error::InvalidTheme)?;

        // Helper to find the text color of a given name
        let fg_color = |name: &str| -> Result<Option<Style>, Error> {
            if let Some(value) = root.get(name) {
                match value {
                    Value::String(reference) => {
                        if let Some(Value::String(color)) = palette.get(reference) {
                            return Ok(Some(Style::from(color)));
                        }
                    }
                    Value::Table(table) => {
                        let mut style = Self::referenced_color(table, palette, "fg")?;

                        if let Some(Value::Array(modifiers)) = table.get("modifiers") {
                            for modifier in modifiers {
                                if let Value::String(modifier) = modifier {
                                    if modifier == "italic" {
                                        style.is_italic = true;
                                    } else if modifier == "bold" {
                                        style.is_bold = true;
                                    }
                                }
                            }
                        }

                        return Ok(Some(style));
                    }
                    _ => {}
                }
            }

            Ok(None)
        };

        let mut style_map = HashMap::default();

        for (index, name) in supported_highlight_names.iter().enumerate() {
            if let Some(style) = fg_color(name)? {
                style_map.insert(index, style);
            }
        }

        let background = match root.get("ui.background") {
            Some(Value::Table(table)) => Self::referenced_color(table, palette, "bg")?,
            _ => Style::from(&"#000".to_string()),
        };

        let foreground = fg_color("ui.text")?.unwrap_or_else(|| Style::from(&"#fff".to_string()));

        Ok(Self {
            style_map,
            foreground,
            background,
            supported_highlight_names,
        })
    }

    /// Simple helper to get the color behind a reference
    /// Exemple when reading this line in TOML file
    // "constant" = "peach"
    // we want to get the color behind "peach" in the palette of color
    // -> peach = "#ef9f76"
    fn referenced_color(table: &Table, palette: &Value, name: &str) -> Result<Style, Error> {
        if let Some(Value::String(reference)) = table.get(name) {
            if let Some(Value::String(color)) = palette.get(reference) {
                return Ok(Style::from(color));
            }
        }

        Err(Error::InvalidColorReference(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::{env::current_dir, fs::read_to_string};

    use super::Theme;

    #[test]
    fn test_can_load_catppuccin_latte_toml_theme() {
        let content = read_to_string(
            current_dir()
                .unwrap()
                .join("src/theming/default/catppuccin_latte.toml"),
        )
        .unwrap();
        let theme = Theme::from_helix(
            &content,
            vec!["variable".to_string(), "function".to_string()],
        )
        .unwrap();

        // See line with: text = "#4c4f69"
        assert_eq!(theme.foreground.color, "#4c4f69");
    }
}
