use std::collections::HashMap;
use std::io::{self, Write};

// TODO: this is a POC implementation of integrating TreeSitter highlighting (inkjet) with Comrak
use comrak::{adapters::SyntaxHighlighterAdapter, html};
use inkjet::{formatter, Highlighter, Language};

pub struct TreeSitterHighlighter {}

impl TreeSitterHighlighter {
    fn find_syntax_by_token(&self, token: &str) -> Option<Language> {
        Some(match token {
            "c" => Language::C,
            "bash" | "sh" | "shell" => Language::Bash,
            _ => return None,
        })
    }
}

// This is based on Syntect integration
// https://docs.rs/comrak/latest/src/comrak/plugins/syntect.rs.html#71-133
impl SyntaxHighlighterAdapter for TreeSitterHighlighter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        match self.find_syntax_by_token(lang.unwrap_or_default()) {
            Some(syntax) => {
                let mut highlighter = Highlighter::new();
                let string = highlighter
                    .highlight_to_string(syntax, &formatter::Html, code)
                    .unwrap_or("oups.".to_string());
                output.write_all(string.as_bytes())
            }
            None => output.write_all(code.as_bytes()),
        }
    }

    fn write_pre_tag(
        &self,

        output: &mut dyn Write,

        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        let _ = html::write_opening_tag(output, "pre", attributes);
        Ok(())
    }

    fn write_code_tag(
        &self,

        output: &mut dyn Write,

        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        html::write_opening_tag(output, "code", attributes)
    }
}
