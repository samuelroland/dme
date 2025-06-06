/// Previewable implementation via a Comrak based Markdown parser
use super::preview::{Html, Previewable};
use super::tree_sitter::TreeSitterHighlighter;

pub struct ComrakParser {}

impl Previewable for ComrakParser {
    fn to_html() -> Html {
        todo!()
    }
}

use std::collections::HashMap;
use std::io::{self, Write};

// TODO: this is a POC implementation of integrating TreeSitter highlighting (inkjet) with Comrak
use comrak::{adapters::SyntaxHighlighterAdapter, html};

// This is based on Syntect integration
// https://docs.rs/comrak/latest/src/comrak/plugins/syntect.rs.html#71-133
impl SyntaxHighlighterAdapter for ComrakParser {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        match self.find_syntax_by_token(lang.unwrap_or_default()) {
            Some(syntax) => {
                output.write_all(TreeSitterHighlighter::new(lang)?.highlight(code).as_bytes())
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
