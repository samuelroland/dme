use std::{borrow::Cow, path::PathBuf};

use ammonia::{Builder, UrlRelativeEvaluate};
use maplit::hashset;

use crate::theming::{renderer::Renderer, theme::Theme};

#[derive(Eq, Debug, PartialEq)]
pub struct Html {
    content: String,
    css_from_theme: String,
    image_rewrite: ImageUrlRewriteMode,
}

impl From<String> for Html {
    fn from(value: String) -> Self {
        Html {
            content: value,
            css_from_theme: String::default(),
            image_rewrite: ImageUrlRewriteMode::None,
        }
    }
}

#[derive(Eq, Debug, PartialEq)]
pub enum ImageUrlRewriteMode {
    /// The default, the URL is not touched
    None,
    // A simple prefix to put before the URL without any other change
    SimplePrefixed(String),
    /// The absolute path built for Tauri, with the asset protocol as prefix.
    /// This is intented to be used by Tauri applications
    TauriFullPath(String),
}

impl Html {
    /// Get a safe HTML version by running ammonia default cleaner with the exception of allow the
    /// class attribute on <code> and <span>, and without cleaning the CSS from the Theme
    pub fn to_safe_html_string(&self) -> String {
        // NOTE: update SECURITY.md if the rules need to change
        let mut cleaner = Builder::default();
        cleaner.add_tag_attributes("code", &["class"]); // authorize the class attribute for <code> because we need to keep highlight names CSS classes
        cleaner.add_tag_attributes("span", &["class"]); // same as for <code>
        cleaner.add_tag_attributes("img", &["src"]);
        // Allow inline SVG, to allow math expressions to be rendered
        // The list of tags has been defined by hand by looking at the minimum for math equations.
        // Some of them might be missing.
        // See https://developer.salesforce.com/docs/platform/lightning-components-security/guide/lws-sanitize-svg.html
        cleaner.add_tag_attributes("p", ["class"]); // to allow the <p class="math-block">
        cleaner.add_tags(["svg", "g", "use", "path", "defs", "symbol"]);
        cleaner.add_tag_attributes(
            "svg",
            [
                "xmlns", // this include xmlns:xlink
                "xlink", "id", "class", "style", "width", "height", "viewBox",
            ],
        );
        cleaner.add_tag_attributes("g", ["class", "transform"]);
        cleaner.add_tag_attributes(
            "use",
            [
                "xlink:href",
                "xlink",
                "href",
                "class",
                "transform",
                "x",
                "y",
                // We do not include them because it seems to have always the default black value for now...
                // "fill",
                // "fill-rule",
            ],
        ); // TODO: make sure xlink:href can be safe ?
           // <path fill="none" stroke="#000" stroke-width=".528" d="M0 1.067h6.292" class="typst-shape"></path>
           //"fill", "stroke" are removed for same reasons as <use>
        cleaner.add_tag_attributes("path", ["d", "class", "stroke-width"]);
        cleaner.add_tag_attributes("defs", ["id"]);
        cleaner.add_tag_attributes("symbol", ["id", "overflow"]);

        cleaner.strip_comments(true);

        // Rewrite the image URLs with a prefix if provided, to adapt to the platform (web needs
        // sometimes a subfolder for images, desktop app needs an absolute path prefix, ...)
        match &self.image_rewrite {
            ImageUrlRewriteMode::None => {}
            ImageUrlRewriteMode::SimplePrefixed(prefix) => {
                struct PurePrefix {
                    prefix: String,
                }
                impl<'a> UrlRelativeEvaluate<'a> for PurePrefix {
                    fn evaluate<'url>(&self, url: &'url str) -> Option<Cow<'url, str>> {
                        // If we have an anchor, we don't want to touch it, because it doesn't need any prefix.
                        // In case of <use href="#abc" > we don't want touch this value.
                        if url.trim().starts_with("#") {
                            Some(Cow::Borrowed(url))
                        } else {
                            let mut copy = self.prefix.clone();
                            copy.push_str(url);
                            Some(Cow::Owned(copy))
                        }
                    }
                }
                cleaner.url_relative(ammonia::UrlRelative::Custom(Box::new(PurePrefix {
                    prefix: prefix.to_owned(),
                })));
            }
            ImageUrlRewriteMode::TauriFullPath(path) => {
                let new_url_schemes = hashset!["asset"];
                cleaner.add_url_schemes(new_url_schemes);

                cleaner.url_relative(ammonia::UrlRelative::Custom(Box::new(TauriPathRewriter {
                    path: path.to_owned(),
                })));
            }
        }

        // Include CSS from the syntax highlighting theme
        let style_markup = format!("<style>{}</style>", self.css_from_theme.as_str());

        format!(
            "{}\n{}",
            style_markup,
            &cleaner.clean(&self.content).to_string()
        )
    }

    /// Push the style generated by a given Theme, this style will not be cleaned !
    pub fn push_style_from_theme(&mut self, theme: &Theme) {
        let renderer = Renderer::new(theme);
        self.css_from_theme.push_str(&renderer.css());
    }

    pub fn set_image_rewrite(mut self, mode: ImageUrlRewriteMode) -> Self {
        self.image_rewrite = mode;
        self
    }
}

struct TauriPathRewriter {
    path: String,
}

/// This is just a reimplementation of Tauri's JavaScript function convertFileSrc()
/// to have a safe Rust only HTML generation without any JavaScript HTML manipulation
/// https://v2.tauri.app/fr/reference/javascript/api/namespacecore/#convertfilesrc
///
/// See implementation to better understand why the tauri_prefix
/// https://github.com/tauri-apps/tauri/blob/18464d9481f4d522c305f21b38be4b906ab41bd5/crates/tauri/scripts/core.js#L13
impl<'a> UrlRelativeEvaluate<'a> for TauriPathRewriter {
    fn evaluate<'url>(&self, url: &'url str) -> Option<std::borrow::Cow<'url, str>> {
        // If we have an anchor, we don't want to touch it, because it doesn't need any prefix.
        // In case of <use href="#abc" > we don't want touch this value.
        if url.trim().starts_with("#") {
            Some(Cow::Borrowed(url))
        } else {
            #[cfg(target_os = "windows")]
            let tauri_prefix = "http://asset.localhost/";
            #[cfg(target_os = "android")]
            let tauri_prefix = "http://asset.localhost/";
            #[cfg(target_os = "linux")]
            let tauri_prefix = "asset://localhost/";
            #[cfg(target_os = "macos")]
            let tauri_prefix = "asset://localhost/";

            let absolute_path = PathBuf::from(&self.path)
                .join(url)
                .into_os_string()
                .to_string_lossy()
                .to_string();
            let encoded_absolute_path = urlencoding::encode(&absolute_path);
            let result = format!("{tauri_prefix}{encoded_absolute_path}");
            Some(Cow::Owned(result))
        }
    }
}

/// A component that will be able to preview a given document into HTML
pub trait Previewable {
    fn to_html(&self, source: &str) -> Html;
}

#[cfg(test)]
mod tests {
    use std::fs::write;

    use crate::preview::{
        comrak::ComrakParser,
        preview::{Html, ImageUrlRewriteMode, Previewable},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_images_path_can_be_left_untouched() {
        let given = "# Sky\n![super sky](sky.png)";
        let result = ComrakParser::new()
            .unwrap()
            .to_html(given)
            .set_image_rewrite(ImageUrlRewriteMode::None)
            .to_safe_html_string();
        assert_eq!(
            result,
            format!("<h1>Sky</h1>\n<p><img src=\"sky.png\" alt=\"super sky\"></p>\n")
        );
    }

    #[test]
    fn test_images_path_can_be_prefixed_and_canonized() {
        let given =
            "# Sky\n![super sky](sky.png)\n![this is not prefixed !](#introduction-to-the-subject)";

        let result = ComrakParser::new()
            .unwrap()
            .to_html(given)
            .set_image_rewrite(ImageUrlRewriteMode::SimplePrefixed(
                "/static/images/".to_string(),
            ))
            .to_safe_html_string();
        let newpath = "/static/images/sky.png";
        assert_eq!(
            result,
            format!("<h1>Sky</h1>\n<p><img src=\"{newpath}\" alt=\"super sky\">\n<img src=\"#introduction-to-the-subject\" alt=\"this is not prefixed !\"></p>\n")
        );
    }

    #[test]
    fn test_images_path_can_be_prefixed_with_absolute_path_for_tauri() {
        let given = "# Sky
![super sky](sky.png)
![super sky on external website](https://great-website.com/sky.png)
![super sky](../../bench/sky.png)
![this is not prefixed !](#introduction-to-the-subject)
![nice path](../images-de-fous/super_Schema$BIEN3joli.png)";

        // TODO: should we support path with spaces ??
        let result = ComrakParser::new()
            .unwrap()
            .to_html(given)
            .set_image_rewrite(ImageUrlRewriteMode::TauriFullPath(
                "/home/sam/report/".to_string(),
            ))
            .to_safe_html_string();

        #[cfg(target_os = "windows")]
        let tauri_prefix = "http://asset.localhost/";
        #[cfg(target_os = "android")]
        let tauri_prefix = "http://asset.localhost/";
        #[cfg(target_os = "linux")]
        let tauri_prefix = "asset://localhost/";
        #[cfg(target_os = "macos")]
        let tauri_prefix = "asset://localhost/";

        assert_eq!(
            result,
            format!(
                "<h1>Sky</h1>
<p><img src=\"{tauri_prefix}%2Fhome%2Fsam%2Freport%2Fsky.png\" alt=\"super sky\">
<img src=\"https://great-website.com/sky.png\" alt=\"super sky on external website\">
<img src=\"{tauri_prefix}%2Fhome%2Fsam%2Freport%2F..%2F..%2Fbench%2Fsky.png\" alt=\"super sky\">
<img src=\"#introduction-to-the-subject\" alt=\"this is not prefixed !\">
<img src=\"{tauri_prefix}%2Fhome%2Fsam%2Freport%2F..%2Fimages-de-fous%2Fsuper_Schema%24BIEN3joli.png\" alt=\"nice path\"></p>
"
            )
        );
    }

    #[test]
    fn test_inline_math_expressions_are_rendered_in_svg() {
        let given = r#"# Circle

Circle area: $A= pi r^2$.

Circular of a circle: $P = 2 pi r$"#;
        let expected = r##"<h1>Circle</h1>
<p>Circle area: <span class="math-inline"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" id="prefix0a" width="52.754" height="10.017" class="typst-frame" style="width:1.31885em;height:.250433em;overflow:visible" viewBox="0 0 39.565 7.513"><g class="typst-group"><use xlink:href="#prefix0b" class="typst-text" transform="matrix(1 0 0 -1 0 7.513)"></use><use xlink:href="#prefix0c" class="typst-text" transform="matrix(1 0 0 -1 11.306 7.513)"></use><use xlink:href="#prefix0d" class="typst-text" transform="matrix(1 0 0 -1 22.92 7.513)"></use><use xlink:href="#prefix0e" class="typst-text" transform="matrix(1 0 0 -1 29.464 7.513)"></use><use xlink:href="#prefix0f" class="typst-text" transform="matrix(1 0 0 -1 34.568 3.52)"></use></g><defs><symbol id="prefix0b" overflow="visible"><path d="M1.507.033C1.705.033 2.387 0 2.596 0c.165 0 .242.088.242.253 0 .099-.066.165-.209.176-.319.011-.473.121-.473.33 0 .099.099.319.308.649.297.495.506.858.649 1.1h2.673c0-.077.011-.231.044-.473.077-.803.121-1.243.121-1.298 0-.209-.242-.308-.737-.308-.209 0-.308-.088-.308-.264q0-.165.198-.165c.264 0 1.1.033 1.364.033C6.688.033 7.469 0 7.689 0c.165 0 .242.088.242.264q0 .165-.297.165c-.308 0-.495.022-.55.055s-.099.132-.11.297l-.66 6.798c-.033.209-.022.297-.253.297-.132 0-.242-.066-.319-.209L1.958 1.32C1.628.77 1.188.473.649.429Q.385.412.385.165C.385.055.451 0 .572 0c.176 0 .759.033.935.033m3.905 6.314.33-3.41H3.377Z"></path></symbol><symbol id="prefix0c" overflow="visible"><path d="M7.678 4.037H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253 0 .132-.121.253-.264.253m0-2.068H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253a.26.26 0 0 1-.264.253"></path></symbol><symbol id="prefix0d" overflow="visible"><path d="M5.764 4.741h-3.63c-.451 0-.847-.176-1.166-.517-.154-.165-.671-.869-.671-1.012.044-.077.044-.143.176-.143.077 0 .143.044.209.143.352.539.803.814 1.342.814h.561C2.332 3.069 1.87 1.892 1.21.495a1 1 0 0 1-.088-.286c0-.22.121-.33.352-.33.209 0 .363.121.462.352.198.627.341 1.111.418 1.441l.605 2.354h1.133c-.297-1.309-.451-2.222-.451-2.739 0-.539.121-1.408.528-1.408.231 0 .484.22.484.451a.8.8 0 0 1-.066.253c-.209.517-.308 1.1-.308 1.771 0 .517.066 1.078.187 1.672h1.199c.385 0 .572.132.572.407 0 .253-.198.308-.473.308"></path></symbol><symbol id="prefix0e" overflow="visible"><path d="M4.796 4.114c0 .462-.451.748-.935.748-.539 0-.99-.253-1.364-.77-.11.429-.484.77-1.001.77-.451 0-.781-.352-1.012-1.067-.11-.363-.165-.572-.165-.638 0-.099.055-.154.176-.154.055 0 .088.011.121.033a.8.8 0 0 1 .099.253c.198.836.451 1.254.748 1.254.187 0 .286-.154.286-.462q0-.215-.165-.891L.957.693C.924.55.858.264.858.209c0-.22.121-.33.352-.33.22 0 .363.11.44.33l.209.792q.182.71.231.924l.341 1.408c.022.088.11.231.253.44.297.418.638.77 1.177.77a.7.7 0 0 0 .341-.077c-.33-.099-.495-.308-.495-.605 0-.286.154-.429.451-.429.363 0 .638.319.638.682"></path></symbol><symbol id="prefix0f" overflow="visible"><path d="M.916 3.265c.247 0 .431.185.431.431 0 .277-.146.424-.43.431.169.362.554.655 1.054.655.678 0 1.124-.509 1.124-1.186 0-.37-.13-.724-.4-1.07q-.196-.266-.3-.37L.57.346C.47.254.485.231.485 0h3.172l.24 1.448h-.323C3.518 1.039 3.456.8 3.387.747 3.349.724 3.111.708 2.657.708H1.348c.515.455.993.863 1.447 1.225.347.27.593.508.747.716q.346.45.346.947c0 .477-.184.855-.562 1.132-.33.254-.747.385-1.24.385-.423 0-.785-.123-1.1-.37-.332-.27-.5-.608-.5-1.024 0-.262.192-.454.43-.454"></path></symbol></defs></svg></span>.</p>
<p>Circular of a circle: <span class="math-inline"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" id="prefix1a" width="53.893" height="10.017" class="typst-frame" style="width:1.34734em;height:.250433em;overflow:visible" viewBox="0 0 40.42 7.513"><g class="typst-group"><use xlink:href="#prefix1b" class="typst-text" transform="matrix(1 0 0 -1 0 7.513)"></use><use xlink:href="#prefix1c" class="typst-text" transform="matrix(1 0 0 -1 11.658 7.513)"></use><use xlink:href="#prefix1d" class="typst-text" transform="matrix(1 0 0 -1 23.271 7.513)"></use><use xlink:href="#prefix1e" class="typst-text" transform="matrix(1 0 0 -1 28.771 7.513)"></use><use xlink:href="#prefix1f" class="typst-text" transform="matrix(1 0 0 -1 35.316 7.513)"></use></g><defs><symbol id="prefix1b" overflow="visible"><path d="M6.105 7.513h-3.52c-.253 0-.374-.011-.374-.253 0-.121.121-.176.363-.176.22 0 .66.033.66-.143a1 1 0 0 0-.044-.198L1.738.902c-.066-.242-.176-.385-.33-.44C1.331.44 1.133.429.792.429.55.429.44.407.44.176.44.055.506 0 .649 0l1.375.033.704-.011c.121 0 .561-.022.704-.022q.264 0 .264.264c0 .11-.121.165-.352.165-.44 0-.66.044-.66.143 0 0 .011.033.033.176l.66 2.684h1.815c.715 0 1.397.22 2.035.649.715.473 1.067 1.056 1.067 1.749 0 1.089-1.034 1.683-2.189 1.683m-.341-.429c.957 0 1.43-.33 1.43-.99 0-.616-.308-1.441-.627-1.727q-.627-.561-1.65-.561H3.443l.726 2.904c.088.374.088.374.55.374Z"></path></symbol><symbol id="prefix1c" overflow="visible"><path d="M7.678 4.037H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253 0 .132-.121.253-.264.253m0-2.068H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253a.26.26 0 0 1-.264.253"></path></symbol><symbol id="prefix1d" overflow="visible"><path d="M2.607 7.326a1.99 1.99 0 0 1-1.441-.594C.759 6.336.55 5.874.55 5.313c0-.374.275-.649.616-.649a.62.62 0 0 1 .605.616.595.595 0 0 1-.616.616c-.033 0-.055 0-.077-.011.209.539.693 1.012 1.386 1.012.902 0 1.408-.781 1.408-1.727 0-.737-.374-1.529-1.122-2.365L.682.473C.539.308.55.319.55 0h4.081l.319 1.98h-.363c-.088-.561-.165-.88-.231-.979-.055-.055-.385-.077-.99-.077H1.529l1.067 1.045c.748.704 1.694 1.463 2.013 2.046q.33.577.33 1.155c0 1.298-1.012 2.156-2.332 2.156"></path></symbol><symbol id="prefix1e" overflow="visible"><path d="M5.764 4.741h-3.63c-.451 0-.847-.176-1.166-.517-.154-.165-.671-.869-.671-1.012.044-.077.044-.143.176-.143.077 0 .143.044.209.143.352.539.803.814 1.342.814h.561C2.332 3.069 1.87 1.892 1.21.495a1 1 0 0 1-.088-.286c0-.22.121-.33.352-.33.209 0 .363.121.462.352.198.627.341 1.111.418 1.441l.605 2.354h1.133c-.297-1.309-.451-2.222-.451-2.739 0-.539.121-1.408.528-1.408.231 0 .484.22.484.451a.8.8 0 0 1-.066.253c-.209.517-.308 1.1-.308 1.771 0 .517.066 1.078.187 1.672h1.199c.385 0 .572.132.572.407 0 .253-.198.308-.473.308"></path></symbol><symbol id="prefix1f" overflow="visible"><path d="M4.796 4.114c0 .462-.451.748-.935.748-.539 0-.99-.253-1.364-.77-.11.429-.484.77-1.001.77-.451 0-.781-.352-1.012-1.067-.11-.363-.165-.572-.165-.638 0-.099.055-.154.176-.154.055 0 .088.011.121.033a.8.8 0 0 1 .099.253c.198.836.451 1.254.748 1.254.187 0 .286-.154.286-.462q0-.215-.165-.891L.957.693C.924.55.858.264.858.209c0-.22.121-.33.352-.33.22 0 .363.11.44.33l.209.792q.182.71.231.924l.341 1.408c.022.088.11.231.253.44.297.418.638.77 1.177.77a.7.7 0 0 0 .341-.077c-.33-.099-.495-.308-.495-.605 0-.286.154-.429.451-.429.363 0 .638.319.638.682"></path></symbol></defs></svg></span></p>"##;

        let result = ComrakParser::new()
            .unwrap()
            .to_html(given)
            .to_safe_html_string();
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_block_math_expressions_are_rendered_in_svg() {
        let given = r#"# Nice matrix
Yop $$G &= mat(
  0, 1;
  0, 0;
  1, 1
)$$"#;

        let expected = r##"<h1>Nice matrix</h1>
<p>Yop </p><p class="math-block"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" id="prefix0a" width="75.77" height="30.507" class="typst-frame" style="width:1.89426em;height:.762667em;overflow:visible" viewBox="0 0 56.828 22.88"><g class="typst-group"><use xlink:href="#prefix0b" class="typst-text" transform="matrix(1 0 0 -1 0 14.19)"></use><use xlink:href="#prefix0c" class="typst-text" transform="matrix(1 0 0 -1 11.702 14.19)"></use><use xlink:href="#prefix0d" class="typst-text" transform="matrix(1 0 0 -1 23.315 14.19)"></use><g class="typst-group"><use xlink:href="#prefix0e" class="typst-text" transform="matrix(1 0 0 -1 32.94 3.496)"></use><use xlink:href="#prefix0e" class="typst-text" transform="matrix(1 0 0 -1 32.94 13.365)"></use><use xlink:href="#prefix0f" class="typst-text" transform="matrix(1 0 0 -1 32.94 23.234)"></use><use xlink:href="#prefix0f" class="typst-text" transform="matrix(1 0 0 -1 42.821 3.496)"></use><use xlink:href="#prefix0e" class="typst-text" transform="matrix(1 0 0 -1 42.821 13.365)"></use><use xlink:href="#prefix0f" class="typst-text" transform="matrix(1 0 0 -1 42.821 23.234)"></use></g><use xlink:href="#prefix0g" class="typst-text" transform="matrix(1 0 0 -1 47.203 14.19)"></use></g><defs><symbol id="prefix0b" overflow="visible"><path d="M3.564-.242c.968 0 1.727.297 2.288.891.066-.143.352-.638.506-.638.055 0 .088.033.11.077s.099.264.198.66l.198.836c.066.253.11.44.143.561.121.484.121.473.715.484.143 0 .209.088.209.264 0 .11-.055.165-.176.165-.209 0-.935-.044-1.144-.033l-1.529.033q-.264 0-.264-.264c0-.099.066-.143.198-.154.627-.033.957-.055.99-.077s.044-.066.044-.121c0-.077-.077-.407-.22-.979C5.61.671 4.796.187 3.773.187c-1.342 0-2.145.891-2.145 2.233 0 .209.022.473.055.77.121.902.715 2.167 1.188 2.75.55.693 1.562 1.386 2.684 1.386 1.144 0 1.705-.869 1.705-2.057 0-.099-.033-.451-.033-.55s.066-.154.209-.154a.4.4 0 0 1 .132.022.7.7 0 0 1 .11.231l.682 2.783c0 .099-.055.154-.165.154-.044 0-.11-.044-.198-.143l-.715-.803c-.429.627-1.034.946-1.815.946-.605 0-1.199-.143-1.804-.418C2.442 6.765 1.551 5.863.979 4.631A4.3 4.3 0 0 1 .55 2.783c0-1.76 1.254-3.025 3.014-3.025"></path></symbol><symbol id="prefix0c" overflow="visible"><path d="M7.678 4.037H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253 0 .132-.121.253-.264.253m0-2.068H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253a.26.26 0 0 1-.264.253"></path></symbol><symbol id="prefix0d" overflow="visible"><path d="M8.778-13.695c.187 0 .275.088.275.275a.26.26 0 0 1-.11.22c-.891.671-1.694 1.727-2.431 3.19C5.203-7.403 4.235-3.74 4.235.011v5.478c0 1.705.198 3.421.583 5.148.682 3.08 2.112 6.545 4.125 8.063.077.055.11.132.11.22 0 .187-.088.275-.275.275a.26.26 0 0 1-.154-.055c-.957-.737-1.87-1.826-2.728-3.289C4.378 13.255 3.047 9.35 3.047 5.489V.011c0-3.63 1.221-7.348 2.585-9.9.902-1.661 1.903-2.915 2.992-3.751a.26.26 0 0 1 .154-.055"></path></symbol><symbol id="prefix0e" overflow="visible"><path d="M3.973 2.456q-.002 1.305-.454 1.987c-.247.385-.74.67-1.332.67q-.372.002-.662-.123C.678 4.635.4 3.65.4 2.456c0-.262.016-.508.039-.747.13-1.008.6-1.863 1.748-1.863.246 0 .47.038.662.123.847.347 1.124 1.31 1.124 2.487m-.878 1.71c.085-.285.131-.817.131-1.61 0-.754-.03-1.286-.1-1.601C3.034.5 2.71.123 2.186.123a.94.94 0 0 0-.562.193c-.238.177-.384.523-.446 1.055-.023.177-.03.57-.03 1.185 0 .747.038 1.263.115 1.54q.196.74.924.74c.462 0 .8-.3.908-.67"></path></symbol><symbol id="prefix0f" overflow="visible"><path d="M2.333 5.113c-.331-.324-.824-.485-1.494-.485v-.339c.455 0 .816.07 1.094.2V.654c0-.1-.008-.161-.031-.192q-.058-.129-.693-.13H.893V0l1.37.03L3.644 0v.331h-.316q-.634.002-.701.131a.5.5 0 0 0-.023.193v4.212c0 .207-.031.246-.27.246"></path></symbol><symbol id="prefix0g" overflow="visible"><path d="M6.578.011v5.478c0 3.63-1.221 7.348-2.585 9.9-.902 1.661-1.903 2.915-2.992 3.751a.26.26 0 0 1-.154.055c-.187 0-.275-.088-.275-.275 0-.088.033-.165.11-.22.891-.671 1.694-1.727 2.431-3.19C4.422 12.903 5.39 9.24 5.39 5.489V.011a23.6 23.6 0 0 0-.583-5.148C4.125-8.217 2.695-11.682.682-13.2a.26.26 0 0 1-.11-.22c0-.187.088-.275.275-.275.055 0 .11.022.154.055.957.737 1.87 1.826 2.728 3.289C5.247-7.755 6.578-3.85 6.578.011"></path></symbol></defs></svg></p><p></p>"##;
        let result = ComrakParser::new()
            .unwrap()
            .to_html(given)
            .to_safe_html_string();
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_line_breaks_in_math_expressions_are_included() {
        let given = r#"$2^3 \ 4+3=7$"#;
        let expected = r##"<p><span class="math-inline"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" id="prefix0a" width="59.679" height="26.148" class="typst-frame" style="width:1.49197em;height:.653693em;overflow:visible" viewBox="0 0 44.759 19.611"><g class="typst-group"><use xlink:href="#prefix0b" class="typst-text" transform="matrix(1 0 0 -1 0 4.1)"></use><use xlink:href="#prefix0c" class="typst-text" transform="matrix(1 0 0 -1 5.5 .108)"></use><use xlink:href="#prefix0d" class="typst-text" transform="matrix(1 0 0 -1 0 18.698)"></use><use xlink:href="#prefix0e" class="typst-text" transform="matrix(1 0 0 -1 7.944 18.698)"></use><use xlink:href="#prefix0f" class="typst-text" transform="matrix(1 0 0 -1 18.947 18.698)"></use><use xlink:href="#prefix0g" class="typst-text" transform="matrix(1 0 0 -1 27.502 18.698)"></use><use xlink:href="#prefix0h" class="typst-text" transform="matrix(1 0 0 -1 39.116 18.698)"></use></g><defs><symbol id="prefix0b" overflow="visible"><path d="M2.607 7.326a1.99 1.99 0 0 1-1.441-.594C.759 6.336.55 5.874.55 5.313c0-.374.275-.649.616-.649a.62.62 0 0 1 .605.616.595.595 0 0 1-.616.616c-.033 0-.055 0-.077-.011.209.539.693 1.012 1.386 1.012.902 0 1.408-.781 1.408-1.727 0-.737-.374-1.529-1.122-2.365L.682.473C.539.308.55.319.55 0h4.081l.319 1.98h-.363c-.088-.561-.165-.88-.231-.979-.055-.055-.385-.077-.99-.077H1.529l1.067 1.045c.748.704 1.694 1.463 2.013 2.046q.33.577.33 1.155c0 1.298-1.012 2.156-2.332 2.156"></path></symbol><symbol id="prefix0c" overflow="visible"><path d="M2.718 2.718c.516.208 1.009.662 1.009 1.317 0 .346-.185.624-.547.824q-.45.255-1.016.254-.567.002-.994-.239c-.346-.192-.523-.462-.523-.808q.002-.44.431-.44c.239 0 .416.186.416.424q-.002.358-.362.416c.223.239.554.362 1.009.362.5 0 .785-.3.785-.793q-.002-.541-.293-.886c-.215-.254-.37-.292-.816-.323-.208-.016-.346.008-.346-.146q.002-.14.223-.139h.408c.662 0 .986-.5.986-1.194C3.088.67 2.78.154 2.133.154Q1.29.152.909.624c.27.03.408.185.408.454a.437.437 0 0 1-.447.447c-.3 0-.454-.154-.454-.47 0-.393.2-.708.593-.932a2.3 2.3 0 0 1 1.14-.277q.702 0 1.231.393.578.427.578 1.108c0 .717-.624 1.21-1.24 1.371"></path></symbol><symbol id="prefix0d" overflow="visible"><path d="M3.883 7.447c-.099 0-.187-.055-.253-.154L.308 2.189v-.396h2.871V.891c0-.198-.044-.33-.121-.385S2.772.429 2.409.429h-.275V0c.319.022.825.033 1.507.033S4.829.022 5.148 0v.429h-.275c-.363 0-.572.022-.649.077s-.121.187-.121.385v.902h1.078v.429H4.103V7.26c0 .11-.077.187-.22.187m-.638-1.364V2.222H.737Z"></path></symbol><symbol id="prefix0e" overflow="visible"><path d="M7.678 3.014H4.543v3.135q0 .264-.264.264t-.264-.264V3.014H.88q-.264 0-.264-.264t.264-.264h3.135V-.649q0-.264.264-.264t.264.264v3.135h3.135q.264 0 .264.264a.27.27 0 0 1-.264.264"></path></symbol><symbol id="prefix0f" overflow="visible"><path d="M3.333 3.883c.726.275 1.408.968 1.408 1.903 0 .473-.231.858-.682 1.155a2.46 2.46 0 0 1-1.353.385c-.495 0-.924-.132-1.309-.385-.429-.286-.649-.66-.649-1.133 0-.363.242-.616.594-.616s.594.253.594.605c0 .363-.209.561-.627.583.286.385.737.583 1.353.583.66 0 .99-.385.99-1.166 0-.462-.088-.847-.253-1.166-.297-.528-.704-.627-1.386-.627-.132-.022-.198-.077-.198-.176 0-.165.077-.165.297-.165h.473c.825 0 1.243-.583 1.243-1.76 0-.935-.341-1.749-1.177-1.749-.715 0-1.243.242-1.562.726a.636.636 0 0 1 .671.649.63.63 0 0 1-.649.649c-.429 0-.649-.22-.649-.671 0-.539.242-.968.726-1.309.429-.297.935-.44 1.496-.44a2.43 2.43 0 0 1 1.639.616c.473.407.704.913.704 1.529 0 1.034-.814 1.749-1.694 1.98"></path></symbol><symbol id="prefix0g" overflow="visible"><path d="M7.678 4.037H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253 0 .132-.121.253-.264.253m0-2.068H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253a.26.26 0 0 1-.264.253"></path></symbol><symbol id="prefix0h" overflow="visible"><path d="M5.225 6.644c.077.099.11.242.11.44H2.673c-.759 0-1.188.044-1.265.143a.5.5 0 0 0-.088.209H.979L.605 5.104h.363c.11.616.198.946.264 1.001.033.033.374.055 1.023.055h2.156L3.245 4.51C2.354 3.245 1.914 1.881 1.914.396c0-.429.176-.638.539-.638s.539.209.539.638v.561c0 1.672.264 2.882.781 3.619Z"></path></symbol></defs></svg></span></p>"##;

        let result = ComrakParser::new()
            .unwrap()
            .to_html(given)
            .to_safe_html_string();
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_math_svg_is_not_sanitized_nor_link_rewritten() {
        // Note: this is simplified SVG to make it shorter here, it doesn't show anything because I broke it...
        let svg = r##"
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" id="a" width="53.893" height="10.017" class="typst-frame" style="width:1.34734em;height:.250433em;overflow:visible" viewBox="0 0 40.42 7.513">
<g class="typst-group">
<use xlink:href="#b" class="typst-text" transform="matrix(1 0 0 -1 0 7.513)"></use>
<use xlink:href="#c" class="typst-text" transform="matrix(1 0 0 -1 11.658 7.513)"></use>
</g>
<defs>
<symbol id="b" overflow="visible">
<path d="M6.105 7.513h-3.52c-. 2.904c.088.374.088.374.55.374Z"></path>
</symbol>
<symbol id="c" overflow="visible">
<path d="M7.678 4.037H.88c-.176 0 1-.264.253"></path>
</symbol>
</defs>
</svg>
"##;

        let html = Html::from(svg.to_string())
            .set_image_rewrite(ImageUrlRewriteMode::SimplePrefixed(
                "https://lxup.org/public/".to_string(),
            ))
            .to_safe_html_string();
        assert_eq!(svg, html);
    }
}
