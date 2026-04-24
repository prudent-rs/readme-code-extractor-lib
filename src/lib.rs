#![doc = include_str!("../README.md")]

// Avoid std:: and use core:: or alloc::  as much as we can.
extern crate alloc;

// On VS Code
// - install https://github.com/ruschaaf/extended-embedded-languages
// - and prefix the raw string with `/*toml*/ ` - see
//   https://github.com/ruschaaf/extended-embedded-languages#embedded-languages
#[doc = r"123\n\n1"]
#[doc = r"123\n\n1( { []} )"]
#[doc = /*toml*/ r#"
    a = "b"
    [xx]
    y = 1
    [dd.xx]
    [[x]]
    h = 1.0
    q = { y = 1. b = 2}
    serde = { version = "1.0.113", features = ["derive"] }
"#]
#[doc = /*toml*/ r#"
a = "b"
[xx]
y = 1
[dd.xx]
h = 1.0
q = { y = 1. b = 2}
"#]

// The following HAS TO have the comment /*toml*/ and the following string opening quote r"
// ON THE SAME LINE.
//
// Otherwise we don't get embedded highlighting with "Embedded Languages" for VS Code
// https://marketplace.visualstudio.com/items?itemName=walteh.embedded-languages-vscode
const _: &str = /*toml*/
    r#"
a = "b"
[xx]
y = 1
[dd.xx]
h = 1.0
q = { y = 1. b = 2}
"#;

const _S1: &str = /*json*/
    r#"
    {"a": "b", "c": [1, 2, 3], "d": 0.333}
"#;

/// Assert that a given trait is dyn-compatible.
macro_rules! assert_dyn_compatible {
    ($trait:path) => {
        const _: () = {
            fn _f(_: &dyn $trait) {}
        };
    };
}

/// Internal/Only for prudent-rs/readme-code-extractor. SemVer-exempt!
pub mod public {
    use alloc::fmt::Debug;
    use core::iter::Peekable;
    use core::str::CharIndices;
    use proc_macro2::{Literal, Span};

    pub mod sealed {
        /// Intentionally NOT public.
        pub(crate) struct TraitParam {}
        pub trait Trait {
            #[allow(private_interfaces)]
            fn _seal(&self, _: &TraitParam);
        }
        assert_dyn_compatible!(Trait);

        /// Intentionally NOT public.
        #[allow(dead_code)]
        pub(crate) struct TraitImpl {}
        impl Trait for TraitImpl {
            fn _seal(&self, _: &TraitParam) {}
        }
    }

    /// Marker-only, no-ops.
    pub mod marker {
        /// Indicate that any `is_***` methods are mutually exclusive (exactly one returns `true`)
        /// and that they indicate `enum`-like.
        pub trait EnumLike: crate::public::sealed::Trait {}
    }

    pub mod config {

        /// Whether we expect a preamble, what kind, and what prefix to inject just before its code.
        pub trait Preamble: crate::public::marker::EnumLike {
            fn is_none(&self) -> bool;
            fn is_copy_verbatim(&self) -> bool;
            fn is_prefixed(&self) -> bool;

            /// If [None], then the preamble is NOT
            /// [crate::private::config::Preamble::ItemsWithPrefix]. If [Some], then the preamble IS
            /// [crate::private::config::Preamble::ItemsWithPrefix], regardless of whether the
            /// &[`str`] is empty or not. If &[`str`] is empty, then it's the same as if
            /// [Preamble::is_copy_verbatim] was `true`.
            fn prefix(&self) -> Option<&str>;
        }
        assert_dyn_compatible!(Preamble);

        pub mod headers {
            pub trait Inserts: crate::public::sealed::Trait {
                // - NOT returning an [Iterator], because [Iterator]
                //   - can NOT be `Box`-ed as Box<&dyn Iterator<Item = &'a str>>`, because Iterator
                //     trait is NOT dyn compatible.
                //   - would require us to export a custom Iterator type.
                // - NOT returning `impl Iterator<Item = &'a str>`, because then this trait would
                //   NOT be dyn-compatible. (That doesn't matter with the current design, but it
                //   would matter if we use &dyn or Box<dyn ...>.)
                // - A slice is more flexible/useable than an [Iterator]. And it knows its length.
                //fn inserts<'a>(&'a self) -> &'a [&'a str];
                fn inserts<'a>(&self) -> &[&str];

                fn after_insert(&self) -> &str;
            }
            assert_dyn_compatible!(Inserts);
        }

        pub trait Headers: crate::public::sealed::Trait {
            fn prefix_before_insert(&self) -> &str;
            fn inserts(&self) -> Option<&dyn headers::Inserts>;
        }
        assert_dyn_compatible!(Headers);
    }

    pub trait Config: crate::public::sealed::Trait + Debug {
        fn file_path(&self) -> &str;

        /// Before preamble (and it applies even if [config::Preamble::is_none]).
        fn start_prefix(&self) -> &str;
        fn preamble(&self) -> &dyn config::Preamble;

        fn ordinary_code_headers(&self) -> Option<&dyn config::Headers>;
        fn ordinary_code_suffix(&self) -> &str;

        fn final_suffix(&self) -> &str;
    }
    assert_dyn_compatible!(Config);
    // ----

    pub trait ConfigContentAndSpan: crate::public::sealed::Trait + Debug {
        fn config_content(&self) -> &str;
        fn span(&self) -> &Span;
    }
    assert_dyn_compatible!(ConfigContentAndSpan);
    pub trait ConfigAndSpan: crate::public::sealed::Trait + Debug {
        fn config(&self) -> &dyn Config;
        fn span(&self) -> &Span;
    }
    assert_dyn_compatible!(ConfigAndSpan);

    pub trait ReadmeLoaded: crate::public::sealed::Trait + Debug {
        fn source_file_content(&self) -> &str;
        fn config(&self) -> &dyn Config;
    }
    assert_dyn_compatible!(ReadmeLoaded);

    pub trait CodeBlock: crate::public::sealed::Trait + Debug {
        fn triple_backtick_suffix(&self) -> &str;
        fn code(&self) -> &str;
    }
    assert_dyn_compatible!(CodeBlock);

    pub trait ReadmeBlock: crate::public::marker::EnumLike + Debug {
        fn is_text(&self) -> bool;
        fn is_code(&self) -> bool;
        fn text(&self) -> Option<&str>;
        fn code(&self) -> Option<&dyn CodeBlock>;
        /// Return code or text content. If `self` holds [CodeBlock], then the returned `&str` is
        /// the same as [CodeBlock::code], that is, excluding any triple backtick suffix.
        fn content(&self) -> &str {
            if let Some(text) = self.text() {
                text
            } else if let Some(code) = self.code() {
                code.code()
            } else {
                unreachable!()
            }
        }
    }
    assert_dyn_compatible!(ReadmeBlock);

    /// Parse a README.md-like input. It's an iterator over [ReadmeBlock].
    ///
    /// We have used a function that called [core::iter::from_fn] and returned a similar iterator.
    /// But that over-complicated the generic signature of [Extracted] to have an `impl
    /// Iterator<Item = ...>` bound. That caused [Extracted]
    /// - to have too verbose `impl`, and
    /// - not to be `&dyn`-compatible.
    #[derive(Debug)]
    pub struct ReadmeBlocksIter<'a> {
        source_content: &'a str,
        pairs: Peekable<CharIndices<'a>>,

        /// Zero-based index of the byte where the current item starts.
        item_start: usize,

        /// Zero-based [usize] index of where the triple backtick suffix ends for the current block.
        ///
        /// [ReadmeBlocksIter::code_triple_backtick_suffix_end] is
        /// - [Some] of [None], or [Some] of [Some], exactly if the current block is a code block
        ///   (that is, if [ReadmeBlocksIter::item_is_code] is `true`).
        ///   - [Some] of [None] if [ReadmeBlocksIter::item_is_code] is `true`, but the triple
        ///     backtick suffix end is not determined yet
        ///   - [Some] of [Some] (incl. `Some(Some(0))`) if the [usize] suffix end is determined
        ///     - [ReadmeBlocksIter::code_triple_backtick_suffix_end] may be [Some] of [Some] even
        ///       if there is NO triple backtick suffix (that is, the suffix is empty) - then the
        ///       [usize] value will be the same as [ReadmeBlocksIter::item_start].
        /// - [None] if [ReadmeBlocksIter::item_is_code] is `false`.
        code_triple_backtick_suffix_end: Option<Option<usize>>,
    }
    impl<'a> ReadmeBlocksIter<'a> {
        /// We start parsing in Markdown/text mode.
        pub(crate) fn new(source_content: &'a str) -> Self {
            Self {
                source_content,
                pairs: source_content.char_indices().peekable(),
                item_start: 0,
                code_triple_backtick_suffix_end: None,
            }
        }
        /// Whether the current item is a code block (rather than a text block).
        pub(crate) fn item_is_code(&self) -> bool {
            matches!(self.code_triple_backtick_suffix_end, Some(_))
        }
    }
    pub type ReadmeBlocksIterPeekable<'a> = Peekable<ReadmeBlocksIter<'a>>;

    /// .peek(), then conditional .next() & drop - only if the peeked value matches the given
    /// pattern.
    ///
    /// Return NOT an iterated value, but bool whether it took & dropped a value, or not.
    macro_rules! peek_and_drop {
        ($iter_mut:expr, $pat:pat) => {{
            if let $pat = $iter_mut.peek() {
                $iter_mut.next();
                true
            } else {
                false
            }
        }};
    }
    impl<'a> Iterator for ReadmeBlocksIter<'a> {
        type Item = crate::private::ReadmeBlock<'a>;

        fn next(&mut self) -> Option<crate::private::ReadmeBlock<'a>> {
            'main: loop {
                if self.code_triple_backtick_suffix_end == Some(None) {
                    // Find end of the triple backtick suffix (if any): Skip until new line.
                    while let Some((byte_idx, c)) = self.pairs.peek() {
                        if *c != '\n' {
                            self.pairs.next();
                            continue;
                        }
                        self.code_triple_backtick_suffix_end = Some(Some(*byte_idx));
                        continue 'main;
                    }
                    break; // end of input
                }

                // Skip leading white space and new lines. @TODO Skip TOML comments.
                while peek_and_drop!(self.pairs, Some((_, ' ' | '\t' | '\n'))) {}
                //if true {panic!("Before triple");}

                if peek_and_drop!(self.pairs, Some((_, '`')))
                    && peek_and_drop!(self.pairs, Some((_, '`')))
                    && peek_and_drop!(self.pairs, Some((_, '`')))
                {
                    //panic!("triple");
                    // Handle immediate end of file - with no trailing new line
                    let peek = self.pairs.peek();
                    let next_block_start = if let Some((idx, _)) = peek {
                        *idx
                    } else {
                        self.source_content.len()
                    };

                    let result = if self.item_is_code() {
                        let code_triple_backtick_suffix_end =
                            self.code_triple_backtick_suffix_end.unwrap_or_else(|| {
                                panic!(
                                    "Internal error: code_triple_backtick_suffix_end should be \
                                     Some(_), but it's None."
                                );
                            });
                        let code_triple_backtick_suffix_end = code_triple_backtick_suffix_end
                            .unwrap_or_else(|| {
                                panic!(
                                    "Internal error: code_triple_backtick_suffix_end is Some(_), \
                                     but it's Some(None). It should be Some(Some(_))."
                                );
                            });

                        crate::private::ReadmeBlock::Code(crate::private::CodeBlock {
                            triple_backtick_suffix: &self.source_content
                                [self.item_start..code_triple_backtick_suffix_end],

                            code: &self.source_content
                                [code_triple_backtick_suffix_end..next_block_start - 3],
                        })
                    } else {
                        crate::private::ReadmeBlock::Text(
                            &self.source_content[self.item_start..next_block_start - 3],
                        )
                    };
                    self.item_start = next_block_start;
                    self.code_triple_backtick_suffix_end = if self.item_is_code() {
                        None
                    } else {
                        Some(None)
                    };
                    //panic!("Result: {result:?}, self: {self:?}");
                    return Some(result);
                } else {
                    self.pairs.next();
                    if self.pairs.peek().is_none() {
                        break;
                    }
                }
                //panic!("AFTER the triple tick peek");
            }
            if self.item_is_code() {
                panic!(
                    "The last code block is not enclosed with three backticks. It started at \n\
                    UTF-8 byte index (zero-based) {}. The rest of the input was: {}",
                    self.item_start,
                    &self.source_content[self.item_start..]
                );
            } else {
                return if self.item_start < self.source_content.len() {
                    let result =
                        crate::private::ReadmeBlock::Text(&self.source_content[self.item_start..]);
                    self.item_start = self.source_content.len();

                    Some(result)
                } else {
                    None
                };
            }
        }
    }
    #[cfg(test)]
    mod readme_blocks_iter_test {
        use crate::private::ReadmeBlock;
        use crate::public::{ReadmeBlock as _, ReadmeBlocksIter};

        #[test]
        fn simplest_one() {
            let iter = ReadmeBlocksIter::new(
                "01 text\n\
                02 text",
            );

            let v = iter.collect::<Vec<_>>();
            assert_eq!(v.len(), 1);

            assert!(matches!(v[0], ReadmeBlock::Text(_)));
            assert_eq!(v[0].text().unwrap().len(), 15);
        }

        #[test]
        fn simplest_two() {
            let iter = ReadmeBlocksIter::new(
                "01 text\n\
                ```\n\
                const _: &str = \"03_code\";\n\
                ```",
            );

            let v = iter.collect::<Vec<_>>();
            assert_eq!(v.len(), 2);

            assert!(matches!(v[0], ReadmeBlock::Text(_)));
            assert_eq!(v[0].text().unwrap().len(), 8);

            assert!(matches!(v[1], ReadmeBlock::Code(_)));
            assert_eq!(v[1].code().unwrap().code().len(), 28);
        }

        #[test]
        fn simplest_three() {
            let iter = ReadmeBlocksIter::new(
                "01 text\n\
                ```\n\
                const _: () = {};\n\
                ```\n\
                text again",
            );

            let v = iter.collect::<Vec<_>>();
            assert_eq!(v.len(), 3);

            assert!(matches!(v[0], ReadmeBlock::Text(_)));
            assert!(matches!(v[0], ReadmeBlock::Text(_)));

            assert!(matches!(v[1], ReadmeBlock::Code(_)));
            assert_eq!(v[1].code().unwrap().code().len(), 19);

            assert!(matches!(v[2], ReadmeBlock::Text(_)));
            assert_eq!(v[2].text().unwrap().len(), 11);
        }

        #[test]
        fn simplest_empty_preamble_text() {
            let iter = ReadmeBlocksIter::new(
                "```\n\
                 const _: &str = \"02_code\";\n\
                 ```\n\
                 text",
            );

            let v = iter.collect::<Vec<_>>();
            assert_eq!(v.len(), 3);

            assert!(matches!(v[1], ReadmeBlock::Code(_)));
            assert_eq!(v[1].code().unwrap().code().len(), 28);
        }

        #[test]
        fn simplest_code_block_is_last() {
            let iter = ReadmeBlocksIter::new(
                "```\n\
                 const _: &str = \"02_code\";\n\
                 ```",
            );

            let v = iter.collect::<Vec<_>>();
            assert_eq!(v.len(), 2);

            assert!(matches!(v[1], ReadmeBlock::Code(_)));
            assert_eq!(v[1].code().unwrap().code().len(), 28);
        }
    }

    pub trait ReadmeExtracted<'a>: crate::public::sealed::Trait + Debug {
        /// Content of the first text block, if any, but only if we do expect a preamble, that is,
        /// if [crate::public::config::Preamble::is_no_preamble] returns `false`.
        ///
        /// If it is [Some], then it must be the "text" variant of [ReadmeBlock], that is, its
        /// [ReadmeBlock::is_code] must return [Some].
        fn preamble_text(&self) -> Option<&dyn ReadmeBlock>;

        /// Content of the first source block, if any, but only if we do expect a preamble, that is,
        /// if [crate::public::config::Preamble::is_no_preamble] returns `false`.
        ///
        /// If it is [Some], then it must be the "code" variant of [ReadmeBlock], that is, its
        /// [ReadmeBlock::is_code] must return [Some].
        fn preamble_code(&self) -> Option<&dyn ReadmeBlock>;

        fn non_preamble_blocks(&mut self) -> &mut ReadmeBlocksIterPeekable<'a>;
    }
    assert_dyn_compatible!(ReadmeExtracted);
    // ------

    #[doc(hidden)]
    #[derive(Debug)]
    pub struct OwnedStringSlice {
        s: String,
        start_incl: usize,
        end_excl: usize,
    }
    impl OwnedStringSlice {
        fn new(s: String, start_incl: usize, end_excl: usize) -> Self {
            Self {
                s,
                start_incl,
                end_excl,
            }
        }
    }
    impl AsRef<str> for OwnedStringSlice {
        fn as_ref(&self) -> &str {
            &self.s[self.start_incl..self.end_excl]
        }
    }

    /// Return string content stored in a given literal. The literal can be
    /// - within quotes "...", or
    /// - a raw string literal `r"...", r#"..."#, r##"..."` (and so on). Do NOT escape - the
    ///   backslash character '\\' in a raw string literal does no escaping.
    ///
    /// There does exist
    /// https://docs.rs/proc-macro2/latest/proc_macro2/struct.Literal.html#method.str_value, but
    /// - enabling it is not trivial (its `procmacro2_semver_exempt` is NOT a feature); and anyway
    /// - it works with `nightly` Rust toolchain only.
    ///
    /// Implementation notes - they matter, so you make an informed decision. We
    /// - call `proc_macro2::Literal`'s
    ///   [`to_string()`](https://docs.rs/proc-macro2/latest/proc_macro2/struct.Literal.html#impl-ToString-for-T)
    /// - that returns `String`, whose **content** is enclosed within quotes `"` and any quotes (and
    ///   special characters) are escaped.
    /// - simply
    ///   - if the string literal starts with a quote '"', remove the leading and trailing quotation
    ///     marks (or, actually, slice it).
    ///   - if the string literal starts with `r", r#", r##", r###"` etc., remove that and the
    ///     appropriate trailing group `", "#, "xx, "xxx` etc. (actually, slice it).
    ///
    /// PANIC is UNLIKELY - it should be only due to an internal error in rustc and/or proc_macro2.
    pub fn string_literal_content(literal: &Literal) -> OwnedStringSlice {
        // Initially it's enclosed by "...", r"...", r#"..."# etc.
        let enclosed = literal.to_string();
        if enclosed.len() < 2 {
            panic!(
                "Expecting an enclosed string literal (at least two bytes), but received: {}",
                enclosed
            );
        }
        // ASCII is common for code scope-only configuration, so applying the initial size same as
        // number of bytes.
        //let mut chars = Vec::with_capacity(enclosed.len());
        //chars.extend(enclosed.chars());
        let mut chars = enclosed.chars();
        let first = chars
            .next()
            .unwrap_or_else(|| panic!("Can't parse the first character of: {enclosed}"));

        let (start_incl, end_excl) = if first == '"' || first == 'r' {
            if first == '"' {
                // ordinary "string literals"
                let last = chars
                    .next_back()
                    .unwrap_or_else(|| panic!("Can't parse the last character of: {enclosed}"));

                assert_eq!(
                    last, '"',
                    "Expecting the last character to be a closing quote '\"', but it's: '{last}'."
                );
                (1, enclosed.len() - 1)
            } else {
                // raw string literals
                let mut num_of_hashes = 0usize;
                while let Some(c) = chars.next() {
                    if c == '#' {
                        num_of_hashes += 1;
                        continue;
                    } else if c == '"' {
                        break;
                    } else {
                        panic!(
                            "Expecting a raw string literal, but surprised by '{c}'. \
                                Whole literal: {enclosed}"
                        );
                    }
                }
                for _ in 0..num_of_hashes {
                    if let Some(c) = chars.next_back() {
                        assert_eq!(
                            c, '#',
                            "Expecting a raw string literal, but it seems not \
                                closed. Surprised by character '{c}' near the end. \
                                Whole literal: {enclosed}"
                        );
                    } else {
                        panic!(
                            "Expecting a raw string literal, but it seems not closed. \
                                Expecting a hash character '#' near the end, but out of \
                                characters. Whole literal: {enclosed}"
                        );
                    }
                }
                if let Some(c) = chars.next_back() {
                    assert_eq!(
                        c, '"',
                        "Expecting a raw string literal, but it seems not closed. \
                            Expecting a quote character '\"' near the end, but \
                            received '{c}' character instead. Whole literal: {enclosed}"
                    );
                } else {
                    panic!(
                        "Expecting a raw string literal, but it seems not closed. \
                            Expecting a quote character '\"' near the end, but out of \
                            characters. Whole literal: {enclosed}"
                    );
                }
                (2 + num_of_hashes, enclosed.len() - 1 - num_of_hashes)
            }
        } else {
            panic!(
                "Expecting a string literal, which would be either \"...\", or r\"...\", \
                    r#\"...\"#, r##\"...\"## (and so on). But received: {enclosed}"
            )
        };

        OwnedStringSlice::new(enclosed, start_incl, end_excl)
    }

    #[doc(hidden)]
    pub fn config_content_and_span(config_content_literal: &Literal) -> impl ConfigContentAndSpan {
        crate::private::ConfigContentAndSpan {
            config_content: crate::public::string_literal_content(config_content_literal),
            span: config_content_literal.span(),
        }
    }

    #[doc(hidden)]
    pub fn config_and_span(
        config_content_and_span: &impl ConfigContentAndSpan,
    ) -> impl ConfigAndSpan {
        let config =
            toml::from_str::<crate::private::Config>(config_content_and_span.config_content());

        match config {
            Ok(config) => crate::private::ConfigAndSpan {
                config,
                span: &config_content_and_span.span(),
            },
            Err(e) => {
                panic!(
                    "Couldn't parse given literal's content as an expected TOML config. Content: \
                    {}\n{:?}",
                    config_content_and_span.config_content(),
                    e
                )
            }
        }
    }

    /// Restriction: We support only files that are in UTF-8 (the content is in UTF-8).
    ///
    /// Return content of the file.
    ///
    /// This function is NOT testable here, because it requires a literal that has [proc_macro2::Span]
    /// (as returned by [proc_macro2::Literal::span]) that comes from a real file and not from a test.
    /// (That is, [proc_macro2::Span::local_file] must return [Some].)
    ///
    /// Therefore, this function is tested as a part of `prudent-rs/readme_code_extractor_proc`.
    pub fn load_file(file_relative_path: impl AsRef<str>, span: &Span) -> String {
        let file_relative_path = file_relative_path.as_ref();

        let cfg_file_path = {
            let invoker_file_path = span.local_file().unwrap_or_else(|| {
                panic!(
                    "Rust source file that invoked \
                    readme_code_extractor_lib::load_file \
                    (through readme_code_extractor::all_by_file! or similar) \
                    macro for file with relative path \
                    {file_relative_path} should have a known location."
                )
            });
            let invoker_parent_dir = invoker_file_path.parent().unwrap_or_else(|| {
                panic!(
                    "Rust source file that invoked readme_code_extractor_lib::load_file \
                    (through readme_code_extractor::all_by_file! or similar) \
                    macro for file with relative path {file_relative_path} \
                    may exist, but we can't get its parent directory.",
                )
            });
            invoker_parent_dir.join(file_relative_path)
        };

        // Error handling is modelling https://doc.rust-lang.org/nightly/src/core/result.rs.html
        // > `fn unwrap_failed`, which invokes `panic!("{msg}: {error:?}");`
        std::fs::read_to_string(&cfg_file_path).unwrap_or_else(|e| {
            let cfg_file_path = cfg_file_path.to_str().unwrap_or("");
            panic!("Expecting a file {cfg_file_path}, but opening it failed: {e:?}",)
        })
    }

    #[doc(hidden)]
    pub fn readme_load(config_and_span: &impl ConfigAndSpan) -> impl ReadmeLoaded {
        crate::private::ReadmeLoaded {
            source_file_content: load_file(
                &config_and_span.config().file_path(),
                config_and_span.span(),
            ),
            config: config_and_span.config(),
        }
    }

    #[doc(hidden)]
    pub fn readme_extract<'a>(
        load: &'a impl crate::public::ReadmeLoaded,
    ) -> impl crate::public::ReadmeExtracted<'a> {
        let mut all_blocks =
            crate::public::ReadmeBlocksIter::new(load.source_file_content()).peekable();

        let (preamble_text, preamble_code) = if load.config().preamble().is_none() {
            (None, None)
        } else {
            let preamble_text = if let Some(block) = all_blocks.peek() {
                if let crate::private::ReadmeBlock::Text(_) = block {
                    all_blocks.next()
                } else {
                    None
                }
            } else {
                None
            };
            let preamble_code = if let Some(block) = all_blocks.peek() {
                if let crate::private::ReadmeBlock::Code(_) = block {
                    all_blocks.next()
                } else {
                    None
                }
            } else {
                None
            };
            (preamble_text, preamble_code)
        };

        crate::private::ReadmeExtracted {
            preamble_text,
            preamble_code,
            non_preamble_blocks: all_blocks,
        }
    }
}

// @TODO conditional compilation - for docs.rs only. See prudent
//
//pub use private as private_documented;

/// Internal/Only for prudent-rs/readme-code-extractor. SemVer-exempt!
///
/// Public only when on docs.rs, so that they get documented. Feature that enables them to be public
/// fails with a compile error if used outside of docs.rs.
pub(crate) mod private {
    use alloc::string::String;
    use proc_macro2::Span;
    use serde::{Deserialize, Serialize};

    pub mod config {
        use serde::{Deserialize, Serialize};

        /// Whether the very first code block is a preamble that needs special handling.
        ///
        /// Intentionally NOT implementing [Clone], as we don't want user code to make copies.
        #[derive(Serialize, Deserialize, Debug)]
        pub enum Preamble<'a> {
            /// No preamble - the very first code block is a non-Preamble block (handled by
            /// injecting any header and/or body strings if set in [crate::private::Config]).
            None,

            /// Expecting a preamble, but no special handling - pass as-is. Any [Headers] and/or
            /// [crate::private::Config::ordinary_code_suffix] will NOT be applied
            /// (prefixed/inserted).
            CopyVerbatim,

            /// Expecting the very first code block to contain "items" ONLY (as per
            /// [`item`](https://lukaswirth.dev/tlborm/decl-macros/minutiae/fragment-specifiers.html#item)
            /// macro capturing variables in declarative macros (ones defined with `macro_rules!`)).
            /// For example, `struct` definitions, `use` or `pub use` imports.
            ///
            /// The [String] value is a prefix injected before each item (located in the same
            /// preamble, that is, the very first code block). Example of a potentially useful
            /// prefix:
            /// - `#[allow(unused_imports)]`, or
            /// - `# #[allow(unused_imports)]` where the leading `#` makes that line
            ///   [hidden](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#hiding-portions-of-the-example)
            ///   in the generated documentation.
            ///
            /// If the [String] value is an empty string, then this is equivalent to
            /// [Preamble::CopyVerbatim].
            Prefixed(&'a str),
        }

        pub mod headers {
            use alloc::vec::Vec;
            use serde::{Deserialize, Serialize};

            #[derive(Serialize, Deserialize, Debug)]
            #[serde(default)]
            pub struct Inserts<'a> {
                /// A list of strings to be injected after the injected
                /// [crate::private::config::Headers::prefix_before_insert], and before the
                /// beginning of the existing code of each non-preamble code block.
                ///
                /// Each string from this list is to be used exactly once, one per each non-preamble
                /// code block. The number of strings in this list has to be the same as the number
                /// of non-preamble code blocks.
                ///
                /// Example of useful inserts: Names of test functions (or parts of such names) to
                /// generate, one per each non-preamble code block.
                pub inserts: Vec<&'a str>,

                /// Content to be injected at the beginning of each non-preamble code block, but
                /// AFTER an insert.
                ///
                /// Example of useful inserts for generating test functions: `() {`.
                pub after_insert: &'a str,
            }
        }

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(default)]
        pub struct Headers<'a> {
            /// Prefix to be injected at the beginning of any non-preamble code block, even before
            /// an insert (if any).
            ///
            /// Example of useful prefix: `#[test] fn test_` for test functions to generate.
            pub prefix_before_insert: &'a str,

            #[serde(borrow)]
            pub inserts: Option<headers::Inserts<'a>>,
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(default)]
    #[non_exhaustive]
    pub struct Config<'a> {
        /// **Relative** path (relative to the directory of Rust source file that invoked the chain
        /// of macros). Defaults to "README.md".
        #[serde(default = "default_config_file_path")]
        pub file_path: &'a str,
        /// @TODO Document (here, and in TOML examples) that prefix_before_preamble
        /// CAN be set & used even IF preamble is set to [config::Preamble::None].
        pub start_prefix: &'a str,

        #[serde(borrow)]
        pub preamble: config::Preamble<'a>,

        #[serde(borrow, default = "default_config_ordinary_code_headers")]
        pub ordinary_code_headers: Option<config::Headers<'a>>,

        /// Suffix to be appended at the end of any non-preamble code block.
        ///
        /// Example of useful inserts for generating test functions: `}`.
        pub ordinary_code_suffix: &'a str,

        pub final_suffix: &'a str,
    }
    fn default_config_file_path() -> &'static str {
        "README.md"
    }
    fn default_config_ordinary_code_headers<'a>() -> Option<config::Headers<'a>> {
        None
    }
    // -----

    #[derive(Debug)]
    pub struct ConfigContentAndSpan {
        pub config_content: crate::public::OwnedStringSlice,
        pub span: Span,
    }

    #[derive(Debug)]
    pub struct ConfigAndSpan<'a> {
        pub config: Config<'a>,
        pub span: &'a Span,
    }

    #[derive(Debug)]
    pub struct ReadmeLoaded<'a> {
        pub source_file_content: String,
        pub config: &'a dyn crate::public::Config,
    }

    #[derive(Debug)]
    pub struct CodeBlock<'a> {
        pub triple_backtick_suffix: &'a str,
        pub code: &'a str,
    }

    #[derive(Debug)]
    pub enum ReadmeBlock<'a> {
        Text(&'a str),
        Code(CodeBlock<'a>),
    }

    #[derive(Debug)]
    pub struct ReadmeExtracted<'a> {
        /// [None] if [crate::public::config::Preamble::is_no_preamble]. But, it may be [None] even
        /// for configurations where preamble is configured. For example: early end of input, or no
        /// text block before the first code block.
        pub preamble_text: Option<ReadmeBlock<'a>>,

        /// [None] if [crate::public::config::Preamble::is_no_preamble]. But, it may be [None] even
        /// for configurations where preamble is configured. For example: early end of input, or no
        /// text block before the first code block.
        pub preamble_code: Option<ReadmeBlock<'a>>,

        pub non_preamble_blocks: crate::public::ReadmeBlocksIterPeekable<'a>,
    }
}

mod trait_impls {
    use crate::{
        public,
        public::sealed::{Trait, TraitParam},
    };
    use proc_macro2::Span;

    impl<'a> Trait for crate::private::config::Preamble<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> Default for crate::private::config::Preamble<'a> {
        fn default() -> Self {
            Self::None
        }
    }
    impl<'a> crate::public::marker::EnumLike for crate::private::config::Preamble<'a> {}
    impl<'a> crate::public::config::Preamble for crate::private::config::Preamble<'a> {
        fn is_none(&self) -> bool {
            matches!(self, Self::None)
        }
        fn is_copy_verbatim(&self) -> bool {
            matches!(self, Self::CopyVerbatim)
        }
        fn is_prefixed(&self) -> bool {
            matches!(self, Self::Prefixed(_))
        }
        fn prefix(&self) -> Option<&str> {
            if let Self::Prefixed(s) = self {
                Some(s)
            } else {
                None
            }
        }
    }

    impl<'a> Trait for crate::private::config::headers::Inserts<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> Default for crate::private::config::headers::Inserts<'a> {
        fn default() -> Self {
            Self {
                inserts: vec![],
                after_insert: "",
            }
        }
    }
    impl<'a> crate::public::config::headers::Inserts for crate::private::config::headers::Inserts<'a> {
        //fn inserts<'s>(&'s self) -> &'s[&'s str] {
        fn inserts(&self) -> &[&str] {
            &self.inserts
        }
        fn after_insert(&self) -> &str {
            &self.after_insert
        }
    }

    impl<'a> Trait for crate::private::config::Headers<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> Default for crate::private::config::Headers<'a> {
        fn default() -> Self {
            if true {
                unreachable!("If this dies, then we don't need default_code_headers")
            }
            Self {
                prefix_before_insert: "",
                inserts: None,
            }
        }
    }

    impl<'a> crate::public::config::Headers for crate::private::config::Headers<'a> {
        fn prefix_before_insert(&self) -> &str {
            &self.prefix_before_insert
        }
        fn inserts(&self) -> Option<&dyn crate::public::config::headers::Inserts> {
            if let Some(inserts) = &self.inserts {
                Some(inserts)
            } else {
                None
            }
        }
    }

    impl<'a> Trait for crate::private::Config<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> Default for crate::private::Config<'a> {
        fn default() -> Self {
            Self {
                file_path: "README.md",

                start_prefix: "",
                preamble: crate::private::config::Preamble::None,

                ordinary_code_headers: None,
                ordinary_code_suffix: "",

                final_suffix: "",
            }
        }
    }
    impl<'a> crate::public::Config for crate::private::Config<'a> {
        fn file_path(&self) -> &str {
            self.file_path
        }
        fn start_prefix(&self) -> &str {
            self.start_prefix
        }
        fn preamble(&self) -> &dyn crate::public::config::Preamble {
            &self.preamble
        }
        fn ordinary_code_headers(&self) -> Option<&dyn crate::public::config::Headers> {
            if let Some(headers) = &self.ordinary_code_headers {
                Some(headers)
            } else {
                None
            }
        }
        fn ordinary_code_suffix(&self) -> &str {
            self.ordinary_code_suffix
        }
        fn final_suffix(&self) -> &str {
            self.final_suffix
        }
    }
    //-----

    impl Trait for crate::private::ConfigContentAndSpan {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl public::ConfigContentAndSpan for crate::private::ConfigContentAndSpan {
        fn config_content(&self) -> &str {
            self.config_content.as_ref()
        }
        fn span(&self) -> &Span {
            &self.span
        }
    }

    impl<'a> Trait for crate::private::ConfigAndSpan<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> public::ConfigAndSpan for crate::private::ConfigAndSpan<'a> {
        fn config(&self) -> &dyn public::Config {
            &self.config
        }
        fn span(&self) -> &Span {
            self.span
        }
    }

    impl<'a> Trait for crate::private::ReadmeLoaded<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> crate::public::ReadmeLoaded for crate::private::ReadmeLoaded<'a> {
        fn source_file_content(&self) -> &str {
            &self.source_file_content
        }
        fn config(&self) -> &dyn crate::public::Config {
            self.config
        }
    }

    impl<'a> Trait for crate::private::CodeBlock<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> crate::public::CodeBlock for crate::private::CodeBlock<'a> {
        fn triple_backtick_suffix(&self) -> &str {
            self.triple_backtick_suffix
        }
        fn code(&self) -> &str {
            self.code
        }
    }

    impl<'a> Trait for crate::private::ReadmeBlock<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> crate::public::marker::EnumLike for crate::private::ReadmeBlock<'a> {}
    impl<'a> crate::public::ReadmeBlock for crate::private::ReadmeBlock<'a> {
        fn is_text(&self) -> bool {
            matches!(self, Self::Text(_))
        }
        fn is_code(&self) -> bool {
            matches!(self, Self::Code(_))
        }
        fn text(&self) -> Option<&str> {
            match self {
                Self::Text(s) => Some(*s),
                Self::Code(_) => None,
            }
        }
        fn code(&self) -> Option<&dyn crate::public::CodeBlock> {
            match self {
                Self::Code(b) => Some(b),
                Self::Text(_) => None,
            }
        }
    }

    impl<'a> Trait for crate::private::ReadmeExtracted<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> crate::public::ReadmeExtracted<'a> for crate::private::ReadmeExtracted<'a> {
        fn preamble_text(&self) -> Option<&dyn crate::public::ReadmeBlock> {
            //self.preamble_text.as_ref()
            match &self.preamble_text {
                Some(preamble_text) => Some(preamble_text),
                None => None,
            }
        }
        fn preamble_code(&self) -> Option<&dyn crate::public::ReadmeBlock> {
            match &self.preamble_code {
                Some(preamble_code) => Some(preamble_code),
                None => None,
            }
        }

        fn non_preamble_blocks(&mut self) -> &mut crate::public::ReadmeBlocksIterPeekable<'a> {
            &mut self.non_preamble_blocks
        }
    }
}

// ------
/// Internal, used between crates `readme-code-extractor-lib` and `readme-code-extractor-proc` and
/// `readme-code-extractor` to assure that they're of the same version.
pub const fn is_exact_version(expected_version: &'static str) -> bool {
    matches!(expected_version.as_bytes(), b"0.0.1")
}

/// No need to be public.
const _ASSERT_VERSION: () = {
    if !crate::is_exact_version(env!("CARGO_PKG_VERSION")) {
        panic!(
            "prudent-rs/readme-code-extractor-lib has its function is_exact_version() out of date."
        );
    }
};

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use proc_macro2::Literal;

    #[test]
    fn string_literal_constructor() {
        let content = "ordinary literal";
        let literal = Literal::string(content);
        assert_eq!(
            crate::public::string_literal_content(&literal).as_ref(),
            content
        );
    }

    #[test]
    fn string_literal_from_str_ordinary() {
        let content = "ordinary literal";

        let enclosed = format!("\"{content}\"");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(
            crate::public::string_literal_content(&literal).as_ref(),
            content
        );
    }

    #[test]
    fn string_literal_from_str_raw_0() {
        let content = "ordinary literal";

        let enclosed = format!("r\"{content}\"");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(
            crate::public::string_literal_content(&literal).as_ref(),
            content
        );
    }

    #[test]
    fn string_literal_from_str_raw_1() {
        let content = "ordinary literal";

        let enclosed = format!("r#\"{content}\"#");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(
            crate::public::string_literal_content(&literal).as_ref(),
            content
        );
    }

    #[test]
    fn string_literal_from_str_raw_2() {
        let content = "ordinary literal";

        let enclosed = format!("r##\"{content}\"##");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(
            crate::public::string_literal_content(&literal).as_ref(),
            content
        );
    }

    /* @TODO move to proc macro test:
    #[test]
    fn load_file_() {
        let literal = Literal::string("tests/file_1.txt");
        let file_content = crate::public::load_file(
            crate::public::string_literal_content(&literal),
            &literal.span(),
        );
        assert_eq!(file_content, "Hi from file_1.txt");
    }
    */
}
