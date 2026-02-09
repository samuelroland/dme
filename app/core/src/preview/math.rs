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
/// compared to <1ms to render a given equation to SVG
pub struct MathRenderer {
    typst: Mutex<TypstWrapperWorld>,
    id_prefix_counter: AtomicU64,
    // Remember all generated equations for faster refresh
    // TODO: manage memory release after some time
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
            println!("Cache HIT for {}", exp.replace("\n", ""));
            return Ok(cached.clone());
        } else {
            println!("Cache miss for {}", exp.replace("\n", ""));
        }

        let prefix = "#set page(height: auto, width: auto, margin: 0pt)";
        let page_with_settings = format!("{prefix}\n${exp}$");
        let mut typst = self.typst.lock().unwrap();
        typst.set_source(page_with_settings);

        let document: PagedDocument = typst::compile(&*typst).output.map_err(|e| {
            e.iter()
                .map(|e| format!("{}: {}", e.message, e.hints.join("\n")))
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

    use crate::preview::math::MathRenderer;

    #[test]
    fn test_valid_math_expression_in_typst_can_be_rendered() {
        let given = "P = 2 pi r";
        let expected = "";

        let renderer = MathRenderer::init();
        let result = renderer.convert_math_expression_into_svg(given).unwrap();
        // println!("Default size: {}", result.bytes().len());
        //
        // println!("Optimized size: {}", result.bytes().len());
        //
        std::fs::write("/tmp/test.svg", &result).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_invalid_math_expression_in_typst_generate_useful_error() {
        let given = "2blabla + pi";
        let expected_error = "unknown variable: blabla: if you meant to display multiple letters as is, try adding spaces between each letter: `b l a b l a`\nor if you meant to display this as text, try placing it in quotes: `\"blabla\"`";

        let renderer = MathRenderer::init();
        let result = renderer.convert_math_expression_into_svg(given);
        if let Err(a) = result {
            assert_eq!(a, expected_error);
        } else {
            panic!("oups")
        }
    }
}
