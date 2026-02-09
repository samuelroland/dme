// This math module provides a way to convert Math expressions into SVG text
// To allow for backend only HTML rendering, it currently only supports the Typst syntax
// See more about the Typst syntax on https://typst.app/docs/reference/math/
// This is based upon https://github.com/tfachmann/typst-as-library/ released under Apache-2.0
// But simplified
// - to focus on math rendering and remove any external file loading (images, fonts, ...)
// - remove some dependencies to reduce the build time
// - remove external package support

use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use mini_moka::sync::Cache;
use oxvg_ast::{parse::roxmltree::parse, serialize::Node as _, visitor::Info};
use oxvg_optimiser::{Jobs, PrefixIds};
use std::convert::TryInto;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::introspection::Introspector;
use typst::layout::{Abs, PagedDocument};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};
use typst_kit::fonts::{FontSearcher, FontSlot};

/// General Math abstraction that should be created once and kept in memory for all future use
/// Keeping in memory is useful to avoid repeating the cost of starting the Typst world which can be 60ms
/// compared to <1ms to render a given equation to SVG. It also contains a cache of the rendered SVG.
pub struct MathRenderer {
    typst: Mutex<TypstWrapperWorld>,
    id_prefix_counter: AtomicU64,
    // Remember most generated equations SVG content for faster refresh
    cache: Cache<String, String>,
}
impl MathRenderer {
    pub fn init() -> Self {
        // See Example: Size Aware Eviction in https://crates.io/crates/mini-moka
        let cache = Cache::builder()
            // A weigher closure takes &K and &V and returns a u32 representing the
            // relative size of the entry. Here, we use the byte length of the value
            // String as the size.
            .weigher(|_key, value: &String| -> u32 { value.len().try_into().unwrap_or(u32::MAX) })
            // This cache will hold up to 500MiB of values.
            .max_capacity(500 * 1024 * 1024)
            .build();
        MathRenderer {
            typst: Mutex::new(TypstWrapperWorld::new()),
            id_prefix_counter: AtomicU64::new(0),
            cache,
        }
    }

    pub fn convert_math_expression_into_svg(&self, exp: &str) -> Result<String, String> {
        let maybe_cached_svg = self.cache.get(&exp.to_string());
        if let Some(cached) = maybe_cached_svg {
            // println!("Cache HIT for {}", exp.replace("\n", ""));
            return Ok(cached.clone());
        } else {
            // println!("Cache miss for {}", exp.replace("\n", ""));
        }

        let prefix = "#set page(height: auto, width: auto, margin: 0pt)";
        let page_with_settings = format!("{prefix}\n${exp}$");
        let mut typst = self.typst.lock().unwrap();
        typst.set_source(page_with_settings);

        let document: PagedDocument = typst::compile(&*typst).output.map_err(|e| {
            e.iter()
                .map(|e| {
                    format!(
                        "Typst parsing error: ${}$\n{}\n{}",
                        exp.trim(),
                        e.message,
                        e.hints.join("\n")
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        })?;
        drop(typst);

        // typst_svg::svg_merged(&document, Abs::pt(2.0))
        let unoptimized_svg = typst_svg::svg_html_frame(
            &document.pages.first().unwrap().frame,
            Abs::pt(30.),
            Some("math"),
            &[],
            &Introspector::default(),
        );

        // Optimize SVG to drastically reduce the size. This is very visible with the floating precision reduced to 3 decimals.
        let maybe_optimized_svg = parse(&unoptimized_svg, |dom, allocator| {
            let mut jobs = Jobs::default();
            let prefixer = PrefixIds {
                // TODO: refactor this temporary hack using delimitor instead of prefix, because oxvg_optimiser::jobs::PrefixGenerator::Prefix("salut".to_string()), is not public...
                // Fixing this would help to be able to change #prefix6b into p6b which is shorter...
                delim: self.get_next_prefix_id().to_string(),
                prefix_ids: true,
                prefix_class_names: false,
                ..PrefixIds::default()
            };
            jobs.prefix_ids = Some(prefixer);
            jobs.run(dom, &Info::new(allocator)).unwrap();
            dom.serialize()
                .expect("DOM serialization has failed during SVG optimisation")
        });
        let final_svg = match maybe_optimized_svg {
            Ok(optimized) => optimized,
            Err(e) => {
                eprintln!("{e}");
                unoptimized_svg
            }
        };
        self.cache.insert(exp.to_string(), final_svg.clone());
        Ok(final_svg)
    }

    fn get_next_prefix_id(&self) -> u64 {
        self.id_prefix_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

// TYPST IMPLEMENTATION

/// Main interface that determines the environment for Typst.
struct TypstWrapperWorld {
    /// The content of a source.
    source: Source,

    /// The standard library.
    library: LazyHash<Library>,

    /// Metadata about all known fonts.
    fonts: Vec<FontSlot>,

    /// Metadata about all known fonts.
    book: LazyHash<FontBook>,

    /// Datetime.
    time: time::OffsetDateTime,
}

impl TypstWrapperWorld {
    pub fn new() -> Self {
        // Note: .include_system_fonts(false) -> means around 3s of time gain on my machine. Embedded fonts seems to be enough on the rendering look.
        let fonts = FontSearcher::new().include_system_fonts(false).search();
        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            source: Source::detached("".to_string()),
            time: time::OffsetDateTime::now_utc(),
        }
    }
}

impl TypstWrapperWorld {
    /// Define the source text to be rendered
    pub fn set_source(&mut self, source: String) {
        self.source = Source::detached(source);
    }
}

/// This is the interface we have to implement such that `typst` can compile it.
///
/// I have tried to keep it as minimal as possible
impl typst::World for TypstWrapperWorld {
    /// Standard library.
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// Metadata about all known Books.
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    /// Accessing the main source file.
    fn main(&self) -> FileId {
        self.source.id()
    }

    /// Accessing a specified source file (based on `FileId`).
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::AccessDenied) // generic error to block all external source file access
        }
    }

    /// Accessing a specified file (non-file).
    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::AccessDenied) // generic error to block all external non source file access
    }

    /// Accessing a specified font per index of font book.
    fn font(&self, id: usize) -> Option<Font> {
        self.fonts[id].get()
    }

    /// Get the current date.
    ///
    /// Optionally, an offset in hours is given.
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;

    use crate::preview::math::MathRenderer;

    #[test]
    fn test_valid_math_expression_in_typst_can_be_rendered() {
        let given = "P = 2 pi r";
        let expected = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" id="prefix0a" width="53.893" height="10.017" class="typst-frame" style="width:1.34734em;height:.250433em;overflow:visible" viewBox="0 0 40.42 7.513"><g class="typst-group"><use xlink:href="#prefix0b" fill="#000" class="typst-text" transform="matrix(1 0 0 -1 0 7.513)"/><use xlink:href="#prefix0c" fill="#000" class="typst-text" transform="matrix(1 0 0 -1 11.658 7.513)"/><use xlink:href="#prefix0d" fill="#000" class="typst-text" transform="matrix(1 0 0 -1 23.271 7.513)"/><use xlink:href="#prefix0e" fill="#000" class="typst-text" transform="matrix(1 0 0 -1 28.771 7.513)"/><use xlink:href="#prefix0f" fill="#000" class="typst-text" transform="matrix(1 0 0 -1 35.316 7.513)"/></g><defs><symbol id="prefix0b" overflow="visible"><path d="M6.105 7.513h-3.52c-.253 0-.374-.011-.374-.253 0-.121.121-.176.363-.176.22 0 .66.033.66-.143a1 1 0 0 0-.044-.198L1.738.902c-.066-.242-.176-.385-.33-.44C1.331.44 1.133.429.792.429.55.429.44.407.44.176.44.055.506 0 .649 0l1.375.033.704-.011c.121 0 .561-.022.704-.022q.264 0 .264.264c0 .11-.121.165-.352.165-.44 0-.66.044-.66.143 0 0 .011.033.033.176l.66 2.684h1.815c.715 0 1.397.22 2.035.649.715.473 1.067 1.056 1.067 1.749 0 1.089-1.034 1.683-2.189 1.683m-.341-.429c.957 0 1.43-.33 1.43-.99 0-.616-.308-1.441-.627-1.727q-.627-.561-1.65-.561H3.443l.726 2.904c.088.374.088.374.55.374Z"/></symbol><symbol id="prefix0c" overflow="visible"><path d="M7.678 4.037H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253 0 .132-.121.253-.264.253m0-2.068H.88c-.176 0-.264-.088-.264-.253s.088-.253.264-.253h6.798c.176 0 .264.088.264.253a.26.26 0 0 1-.264.253"/></symbol><symbol id="prefix0d" overflow="visible"><path d="M2.607 7.326a1.99 1.99 0 0 1-1.441-.594C.759 6.336.55 5.874.55 5.313c0-.374.275-.649.616-.649a.62.62 0 0 1 .605.616.595.595 0 0 1-.616.616c-.033 0-.055 0-.077-.011.209.539.693 1.012 1.386 1.012.902 0 1.408-.781 1.408-1.727 0-.737-.374-1.529-1.122-2.365L.682.473C.539.308.55.319.55 0h4.081l.319 1.98h-.363c-.088-.561-.165-.88-.231-.979-.055-.055-.385-.077-.99-.077H1.529l1.067 1.045c.748.704 1.694 1.463 2.013 2.046q.33.577.33 1.155c0 1.298-1.012 2.156-2.332 2.156"/></symbol><symbol id="prefix0e" overflow="visible"><path d="M5.764 4.741h-3.63c-.451 0-.847-.176-1.166-.517-.154-.165-.671-.869-.671-1.012.044-.077.044-.143.176-.143.077 0 .143.044.209.143.352.539.803.814 1.342.814h.561C2.332 3.069 1.87 1.892 1.21.495a1 1 0 0 1-.088-.286c0-.22.121-.33.352-.33.209 0 .363.121.462.352.198.627.341 1.111.418 1.441l.605 2.354h1.133c-.297-1.309-.451-2.222-.451-2.739 0-.539.121-1.408.528-1.408.231 0 .484.22.484.451a.8.8 0 0 1-.066.253c-.209.517-.308 1.1-.308 1.771 0 .517.066 1.078.187 1.672h1.199c.385 0 .572.132.572.407 0 .253-.198.308-.473.308"/></symbol><symbol id="prefix0f" overflow="visible"><path d="M4.796 4.114c0 .462-.451.748-.935.748-.539 0-.99-.253-1.364-.77-.11.429-.484.77-1.001.77-.451 0-.781-.352-1.012-1.067-.11-.363-.165-.572-.165-.638 0-.099.055-.154.176-.154.055 0 .088.011.121.033a.8.8 0 0 1 .099.253c.198.836.451 1.254.748 1.254.187 0 .286-.154.286-.462q0-.215-.165-.891L.957.693C.924.55.858.264.858.209c0-.22.121-.33.352-.33.22 0 .363.11.44.33l.209.792q.182.71.231.924l.341 1.408c.022.088.11.231.253.44.297.418.638.77 1.177.77a.7.7 0 0 0 .341-.077c-.33-.099-.495-.308-.495-.605 0-.286.154-.429.451-.429.363 0 .638.319.638.682"/></symbol></defs></svg>"##;

        let renderer = MathRenderer::init();
        let result = renderer.convert_math_expression_into_svg(given).unwrap();
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_invalid_math_expression_in_typst_generate_useful_error() {
        let given = "2blabla + pi";
        let expected_error = "Typst parsing error: $2blabla + pi$
unknown variable: blabla
if you meant to display multiple letters as is, try adding spaces between each letter: `b l a b l a`
or if you meant to display this as text, try placing it in quotes: `\"blabla\"`";

        let renderer = MathRenderer::init();
        let result = renderer.convert_math_expression_into_svg(given);
        if let Err(a) = result {
            assert_eq!(a, expected_error);
        } else {
            panic!("oups")
        }
    }
}
