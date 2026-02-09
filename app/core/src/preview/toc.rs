use std::ops::Deref;

use comrak::{nodes::NodeValue, parse_document, Arena, Options};
use serde::{Deserialize, Serialize};

use crate::preview::comrak::{FRONT_MATTER_DELIMITER, HEADER_IDS_SECURITY_PREFIX};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct TocHeading {
    text: String,
    level: u8,
    id: String,
}

/// Generate a dynamic Table of Content (TOC) from a given Markdown as a vec of TocHeading.
/// This also generate headers IDs by reusing comrak::html::Anchorizer. This makes sure we have the same logic on both sides.
pub fn generate_table_of_content(markdown_content: &str) -> Vec<TocHeading> {
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some(FRONT_MATTER_DELIMITER.into());

    // Go down the AST to search for all headings
    let arena = Arena::new();
    let root = parse_document(&arena, markdown_content, &options);
    let mut headings = Vec::new();
    let mut anchorizer = comrak::html::Anchorizer::new();
    for node in root.descendants() {
        let node_borrow = &mut node.data.borrow();
        if let NodeValue::Heading(node_heading) = &node_borrow.value {
            let level = node_heading.level;

            // The heading itself has not .text field, we have to go deeper in the tree
            let children_node_maybe_text = node.children().next();
            match children_node_maybe_text {
                Some(children) => {
                    if let NodeValue::Text(text) = &children.data.borrow().deref().value {
                        let text = text.to_string();
                        let id = format!(
                            "{}{}",
                            HEADER_IDS_SECURITY_PREFIX,
                            anchorizer.anchorize(&text)
                        );
                        headings.push(TocHeading { text, level, id });
                    }
                }
                None => todo!(),
            }
        }
    }

    headings
}

#[cfg(test)]
mod tests {
    use crate::preview::toc::{generate_table_of_content, TocHeading};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_toc_can_be_loaded() {
        let content =
            "#      Hey  \n## H2\nhello\n## H2 again\n#### H4 some 😃 brOK!$en title ééààà";
        let result = generate_table_of_content(content);
        let expected = vec![
            TocHeading {
                text: "Hey".into(),
                level: 1,
                id: "h-hey".into(),
            },
            TocHeading {
                text: "H2".into(),
                level: 2,
                id: "h-h2".into(),
            },
            TocHeading {
                text: "H2 again".into(),
                level: 2,
                id: "h-h2-again".into(),
            },
            TocHeading {
                text: "H4 some 😃 brOK!$en title ééààà".into(),
                level: 4,
                id: "h-h4-some--broken-title-ééààà".into(),
            },
        ];
        assert_eq!(result, expected);
    }
}
