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

// The following HAS TO have both the comment /*toml*/ and the following string opening quote r"
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
    use std::collections::HashSet;

    use proc_macro2_diagnostics::Diagnostic;

    pub type MacroResult<T> = Result<T, Diagnostic>;
    pub type MacroDeepResult<T> = Result<T, DeepDiagnostic>;

    #[derive(Clone, Debug)]
    pub struct DeepDiagnostic {
        level: proc_macro2_diagnostics::Level,
        message: String,
    }
    impl DeepDiagnostic {
        // @TODO macro_rules and also generate: pub fn warning, note, help
        pub fn error<T: Into<String>>(message: T) -> Self {
            Self {
                level: proc_macro2_diagnostics::Level::Error,
                message: message.into(),
            }
        }
        // @TODO if implemented in proc_macro2_diagnostics, make it accept MultiSpan:
        //
        // pub fn spanned<S: MultiSpan>(self, s: S) -> Diagnostic
        pub fn spanned(self, span: Span) -> Diagnostic {
            Diagnostic::spanned(span, self.level, self.message)
        }
    }

    // @TODO consider making it a sealed trait
    pub trait MacroResultDeepExt<T> {
        // @TODO if implemented in proc_macro2_diagnostics, make it accept MultiSpan.
        fn spanned(self, span: Span) -> MacroResult<T>;
    }
    impl<T> MacroResultDeepExt<T> for MacroDeepResult<T> {
        fn spanned(self, span: Span) -> MacroResult<T> {
            self.map_err(|deep_err| deep_err.spanned(span))
        }
    }

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
            pub trait Tags: crate::public::sealed::Trait {
                /// Unique tags - if any, then it's exactly one tag per code block.
                ///
                /// - NOT returning `impl Iterator<Item = &'a str>`, because then this trait would
                ///   NOT be dyn-compatible.
                /// - A slice is more flexible/useable than an [Iterator]. And it knows its length.
                fn tags(&self) -> &[&str];

                fn after_tag(&self) -> &str;
            }
            assert_dyn_compatible!(Tags);
        }

        pub trait Headers: crate::public::sealed::Trait {
            fn prefix_before_tag(&self) -> &str;
            fn tags(&self) -> Option<&dyn headers::Tags>;
        }
        assert_dyn_compatible!(Headers);
    }

    pub trait Config: crate::public::sealed::Trait + Debug {
        /// Markdown file path, relative to where the relevant readme_code_extractor's macro is
        /// being used.
        fn markdown_file_local_path(&self) -> &str;

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
        fn span(&self) -> Span;
    }
    assert_dyn_compatible!(ConfigContentAndSpan);
    pub trait ConfigAndSpan: crate::public::sealed::Trait + Debug {
        fn config(&self) -> &dyn Config;
        fn span(&self) -> Span;
    }
    assert_dyn_compatible!(ConfigAndSpan);

    pub trait ReadmeLoaded: crate::public::sealed::Trait + Debug {
        // NOT necessary for data flow, but it makes error reporting easier.
        //fn span(&self) -> &Span;
        fn markdown_file_content(&self) -> &str;
        fn config(&self) -> &dyn Config;
        /// See [Config::markdown_file_local_path].
        fn markdown_file_local_path(&self) -> &str;
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
        //span: &'a Span,
        markdown_content: &'a str,
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
        pub(crate) fn new(/*span: &'a Span,*/ markdown_content: &'a str) -> Self {
            Self {
                //span,
                markdown_content,
                pairs: markdown_content.char_indices().peekable(),
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
        type Item = MacroDeepResult<crate::private::ReadmeBlock<'a>>;

        fn next(&mut self) -> Option<MacroDeepResult<crate::private::ReadmeBlock<'a>>> {
            'main: loop {
                if self.code_triple_backtick_suffix_end == Some(None) {
                    // Find end of the triple backtick suffix (if any).
                    while let Some((byte_idx, c)) = self.pairs.peek() {
                        if *c != '\n' {
                            self.pairs.next(); // Skip until new line.
                            continue;
                        }
                        self.code_triple_backtick_suffix_end = Some(Some(*byte_idx));
                        continue 'main;
                    }
                    break; // end of input
                }

                // Skip leading white space and new lines. @TODO Skip TOML comments.
                while peek_and_drop!(self.pairs, Some((_, ' ' | '\t' | '\n'))) {}

                if peek_and_drop!(self.pairs, Some((_, '`')))
                    && peek_and_drop!(self.pairs, Some((_, '`')))
                    && peek_and_drop!(self.pairs, Some((_, '`')))
                {
                    let next_block_start = if let Some((idx, _)) = self.pairs.peek() {
                        *idx
                    } else {
                        // Handle immediate end of file - with no trailing new line
                        self.markdown_content.len()
                    };

                    let result = if self.item_is_code() {
                        let code_triple_backtick_suffix_end =
                            self.code_triple_backtick_suffix_end.unwrap_or_else(|| {
                                panic!(
                                    "Internal error: code_triple_backtick_suffix_end should be \
                                        Some(Some(usize)), but it's None."
                                );
                            });
                        let code_triple_backtick_suffix_end = code_triple_backtick_suffix_end
                            .unwrap_or_else(|| {
                                panic!(
                                    "Internal error: code_triple_backtick_suffix_end is \
                                     Some(None), but it should be Some(Some(usize))."
                                );
                            });

                        crate::private::ReadmeBlock::Code(crate::private::CodeBlock {
                            triple_backtick_suffix: &self.markdown_content
                                [self.item_start..code_triple_backtick_suffix_end],

                            code: &self.markdown_content
                                [code_triple_backtick_suffix_end..next_block_start - 3],
                        })
                    } else {
                        crate::private::ReadmeBlock::Text(
                            &self.markdown_content[self.item_start..next_block_start - 3],
                        )
                    };

                    self.item_start = next_block_start;
                    self.code_triple_backtick_suffix_end = if self.item_is_code() {
                        None
                    } else {
                        Some(None)
                    };

                    return Some(Ok(result));
                } else {
                    self.pairs.next();
                    if self.pairs.peek().is_none() {
                        break;
                    }
                }
            }
            if self.item_is_code() {
                return Some(Err(DeepDiagnostic::error(format!(
                    "The last code block is not enclosed with three backticks. It started at \
                    UTF-8 byte index (indexed from zero) {}. The rest of the input was: {}",
                    self.item_start,
                    &self.markdown_content[self.item_start..]
                ))));
            } else {
                return if self.item_start < self.markdown_content.len() {
                    let result = crate::private::ReadmeBlock::Text(
                        &self.markdown_content[self.item_start..],
                    );
                    self.item_start = self.markdown_content.len();

                    Some(Ok(result))
                } else {
                    None
                };
            }
        }
    }
    #[cfg(test)]
    mod readme_blocks_iter_test {
        use crate::private::ReadmeBlock;
        use crate::public::{
            MacroDeepResult, MacroResult, MacroResultDeepExt, ReadmeBlock as _, ReadmeBlocksIter,
        };
        use core::str::FromStr;
        use proc_macro2::Literal;

        #[test]
        fn simplest_one() -> MacroResult<()> {
            let span = Literal::from_str("0").unwrap().span();
            let iter = ReadmeBlocksIter::new(
                //&span,
                "01 text\n\
                02 text",
            );

            let v = iter.collect::<MacroDeepResult<Vec<_>>>().spanned(span)?;
            assert_eq!(v.len(), 1);

            assert!(matches!(v[0], ReadmeBlock::Text(_)));
            assert_eq!(v[0].text().unwrap().len(), 15);
            Ok(())
        }

        #[test]
        fn simplest_two() -> MacroDeepResult<()> {
            //let span = Literal::from_str("0").unwrap().span();
            let iter = ReadmeBlocksIter::new(
                // /&span,
                "01 text\n\
                ```\n\
                const _: &str = \"03_code\";\n\
                ```",
            );

            let v = iter.collect::<MacroDeepResult<Vec<_>>>()?;
            assert_eq!(v.len(), 2);

            assert!(matches!(v[0], ReadmeBlock::Text(_)));
            assert_eq!(v[0].text().unwrap().len(), 8);

            assert!(matches!(v[1], ReadmeBlock::Code(_)));
            assert_eq!(v[1].code().unwrap().code().len(), 28);
            Ok(())
        }

        #[test]
        fn simplest_three() -> MacroResult<()> {
            let span = Literal::from_str("0").unwrap().span();
            let iter = ReadmeBlocksIter::new(
                //&span,
                "01 text\n\
                ```\n\
                const _: () = {};\n\
                ```\n\
                text again",
            );

            let v = iter.collect::<MacroDeepResult<Vec<_>>>().spanned(span)?;
            assert_eq!(v.len(), 3);

            assert!(matches!(v[0], ReadmeBlock::Text(_)));
            assert!(matches!(v[0], ReadmeBlock::Text(_)));

            assert!(matches!(v[1], ReadmeBlock::Code(_)));
            assert_eq!(v[1].code().unwrap().code().len(), 19);

            assert!(matches!(v[2], ReadmeBlock::Text(_)));
            assert_eq!(v[2].text().unwrap().len(), 11);
            Ok(())
        }

        #[test]
        fn simplest_empty_preamble_text() -> MacroResult<()> {
            let span = Literal::from_str("0").unwrap().span();
            let iter = ReadmeBlocksIter::new(
                //&span,
                "```\n\
                 const _: &str = \"02_code\";\n\
                 ```\n\
                 text",
            );

            let v = iter.collect::<MacroDeepResult<Vec<_>>>().spanned(span)?;
            assert_eq!(v.len(), 3);

            assert!(matches!(v[1], ReadmeBlock::Code(_)));
            assert_eq!(v[1].code().unwrap().code().len(), 28);
            Ok(())
        }

        #[test]
        fn simplest_code_block_is_last() -> MacroResult<()> {
            let span = Literal::from_str("0").unwrap().span();
            let iter = ReadmeBlocksIter::new(
                //&span,
                "```\n\
                 const _: &str = \"02_code\";\n\
                 ```",
            );

            let v = iter.collect::<MacroDeepResult<Vec<_>>>().spanned(span)?;
            assert_eq!(v.len(), 2);

            assert!(matches!(v[1], ReadmeBlock::Code(_)));
            assert_eq!(v[1].code().unwrap().code().len(), 28);
            Ok(())
        }
    }

    pub trait ReadmeExtracted<'a>: crate::public::sealed::Trait + Debug {
        // NOT necessary for data flow, but it makes error reporting easier.
        //fn span(&self) -> &Span;

        /// See [Config::markdown_file_local_path].
        fn markdown_file_local_path(&self) -> &str;

        /// Content of the first text block, if any, but only if we do expect a preamble, that is,
        /// if [crate::public::config::Preamble::is_no_preamble] returns `false`.
        ///
        /// If it is [Some], then it must be the "text" variant of [ReadmeBlock], that is, its
        /// [ReadmeBlock::is_code] must return [Some].
        fn preamble_text(&self) -> Option<&dyn ReadmeBlock>;

        /// Content of the first code block, if any, but only if we do expect a preamble, that is,
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
        fn new_from_string(s: String, start_incl: usize, end_excl: usize) -> Self {
            Self {
                s,
                start_incl,
                end_excl,
            }
        }
        fn new_from_whole_string(s: String) -> Self {
            let end_excl = s.len();
            Self {
                s,
                start_incl: 0,
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
    ///     tags (or, actually, slice it).
    ///   - if the string literal starts with `r", r#", r##", r###"` etc., remove that and the
    ///     appropriate trailing group `", "#, "xx, "xxx` etc. (actually, slice it).
    ///
    /// Parameter `enclosed` is enclosed by "...", or r"...", r#"..."# etc. You can pass [Literal]'s
    /// `to_string()`.
    ///
    /// PANIC is UNLIKELY - it should be only due to an internal error in rustc and/or proc_macro2.
    pub fn string_literal_content(enclosed_owned: &Literal) -> MacroResult<OwnedStringSlice> {
        let span = enclosed_owned.span();
        let enclosed_owned = enclosed_owned.to_string();
        let (start_incl, end_excl) = string_literal_start_end(&enclosed_owned).spanned(span)?;
        Ok(OwnedStringSlice::new_from_string(
            enclosed_owned,
            start_incl,
            end_excl,
        ))
    }

    /// Use inside [string_literal_start_end] and similar.
    macro_rules! some_or_fail{
        ( $span:expr, $option_expr:expr, $( $rest:tt)+ ) => {
            ({
            use ::proc_macro2_diagnostics::SpanDiagnosticExt as _;
            match $option_expr {
                ::core::option::Option::Some(value) => value,
                ::core::option::Option::None => {
                    return ::core::result::Result::Err($span.error(
                        format!(
                            $( $rest )+
                        )
                    ));
                }
            }
            })
        };
    }

    /// Use inside [string_literal_start_end] and similar.
    macro_rules! some_or_fail_deep{
        ( $option_expr:expr, $( $rest:tt)+ ) => {
            match $option_expr {
                ::core::option::Option::Some(value) => value,
                ::core::option::Option::None => {
                    return ::core::result::Result::Err($crate::public::DeepDiagnostic::error(
                        format!(
                            $( $rest )+
                        )
                    ));
                }
            }
        };
    }

    /// Use inside [string_literal_start_end] and similar.
    #[macro_export]
    macro_rules! true_or_fail{
        ( $span:expr, $bool_expr:expr, $( $rest:tt)+ ) => {
            ({
            use ::proc_macro2_diagnostics::SpanDiagnosticExt as _;
            if !$bool_expr {
                return ::core::result::Result::Err($span.error(
                        format!(
                            $( $rest )+
                        )
                    ));
            }
            })
        };
    }

    /// Use inside [string_literal_start_end] and similar.
    #[macro_export]
    macro_rules! true_or_fail_deep{
        ( $bool_expr:expr, $( $rest:tt)+ ) => {
            if !$bool_expr {
                return ::core::result::Result::Err($crate::public::DeepDiagnostic::error(
                        format!(
                            $( $rest )+
                        )
                    ));
            }
        };
    }

    /// Use inside [string_literal_start_end] and similar.
    ///
    /// Pass a formatting string as the first part of the "rest" parameter. The last placeholder
    /// `{}` in the formatting string will be populated with the original error.
    #[macro_export]
    macro_rules! ok_or_fail{
        ( $span:expr, $result_expr:expr, $( $rest:tt)+ ) => {
            ({
            use ::proc_macro2_diagnostics::SpanDiagnosticExt as _;
            match $result_expr {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(err) => {
                    return ::core::result::Result::Err($span.error(
                        format!(
                            $( $rest )+ , err
                        )
                    ));
                }
            }
            })
        };
    }

    /// Use inside [string_literal_start_end] and similar.
    ///
    /// Pass a formatting string as the first part of the "rest" parameter. The last placeholder
    /// `{}` in the formatting string will be populated with the original error.
    #[macro_export]
    macro_rules! ok_or_fail_deep {
        ( $result_expr:expr, $( $rest:tt)+ ) => {
            match $result_expr {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(err) => {
                    return ::core::result::Result::Err($crate::public::DeepDiagnostic::error(
                        format!(
                            $( $rest )+ , err
                        )
                    ));
                }
            }
        };
    }

    pub fn string_literal_start_end(enclosed: &str) -> MacroDeepResult<(usize, usize)> {
        true_or_fail_deep!(
            enclosed.len() > 2,
            "Expecting an enclosed string literal (at least two bytes), but received: {}",
            enclosed
        );

        let mut chars = enclosed.chars();
        let first = some_or_fail_deep! {
            chars.next(),
            "Can't parse the first character of: {enclosed}"

        };

        if first == '"' || first == 'r' {
            if first == '"' {
                // ordinary "string literals"
                let last = some_or_fail_deep!(
                    chars.next_back(),
                    "Can't parse the last character of: {enclosed}"
                );

                true_or_fail_deep!(
                    last == '"',
                    "Expecting the last character to be a closing quote '\"', but it's: '{last}'."
                );
                Ok((1, enclosed.len() - 1))
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
                        return Err(DeepDiagnostic::error(
                            "Expecting a raw string literal, but surprised by '{c}'. \
                                Whole literal: {enclosed}",
                        ));
                    }
                }
                for _ in 0..num_of_hashes {
                    let c = some_or_fail_deep!(
                        chars.next_back(),
                        "Expecting a raw string literal, but it seems not closed. \
                                Expecting a hash character '#' near the end, but out of \
                                characters. Whole literal: {enclosed}"
                    );
                    true_or_fail_deep!(
                        c == '#',
                        "Expecting a raw string literal, but it seems not \
                            closed. Surprised by character '{c}' near the end. \
                            Whole literal: {enclosed}"
                    );
                }
                let c = some_or_fail_deep!(
                    chars.next_back(),
                    "Expecting a raw string literal, but it \
                            seems not closed. \
                            Expecting a quote character '\"' near the end, but out of \
                            characters. Whole literal: {enclosed}"
                );
                true_or_fail_deep!(
                    c == '"',
                    "Internal or unexpected error: Expecting a raw string literal, but it \
                            seems not closed. \
                            Expecting a quote character '\"' near the end, but \
                            received '{c}' character instead. Whole literal: {enclosed}"
                );

                Ok((2 + num_of_hashes, enclosed.len() - 1 - num_of_hashes))
            }
        } else {
            Err(DeepDiagnostic::error(
                "Internal Error: Expecting a string literal, which would be either \"...\", or r\"...\", \
                    r#\"...\"#, r##\"...\"## (and so on). But received: {enclosed}",
            ))
        }
    }

    #[doc(hidden)]
    pub fn config_content_and_span(
        config_content_literal: &Literal,
    ) -> MacroResult<impl ConfigContentAndSpan> {
        Ok(crate::private::ConfigContentAndSpan {
            config_content: crate::public::string_literal_content(config_content_literal)?,
            span: config_content_literal.span(),
        })
    }

    /// Read configuration from a (TOML) file, its path is given as `config_file_path_literal`.
    ///
    /// Return impl [ConfigContentAndSpan], and a path to the TOML config file.
    #[doc(hidden)]
    pub fn config_content_and_span_by_file(
        config_file_path_literal: &Literal,
    ) -> MacroResult<(impl ConfigContentAndSpan, OwnedStringSlice)> {
        let toml_config_file_path =
            crate::public::string_literal_content(config_file_path_literal)?;

        let span = config_file_path_literal.span();
        let config_content = load_file(span, &toml_config_file_path)?;
        let config_content = OwnedStringSlice::new_from_whole_string(config_content);

        Ok((
            crate::private::ConfigContentAndSpan {
                config_content,
                span,
            },
            toml_config_file_path,
        ))
    }

    #[doc(hidden)]
    pub fn config_and_span(
        config_content_and_span: &impl ConfigContentAndSpan,
    ) -> MacroResult<impl ConfigAndSpan> {
        let config =
            toml::from_str::<crate::private::Config>(config_content_and_span.config_content());

        let config = ok_or_fail!(
            config_content_and_span.span(),
            config,
            "Couldn't parse given literal's content as an expected TOML config. Content: \
                    {}\nError:\n{:?}",
            config_content_and_span.config_content()
        );

        if let Some(headers) = config.ordinary_code_headers()
            && let Some(tags) = headers.tags()
        {
            let tags = tags.tags();
            if tags.len() > 0 {
                let mut set = HashSet::<&str>::with_capacity(tags.len());
                set.extend(tags.iter());
                true_or_fail!(
                    config_content_and_span.span(),
                    set.len() == tags.len(),
                    "Since tags were given, they must be unique! However, there are {} tags, but only {} unique subsets of them.",
                    tags.len(),
                    set.len()
                )
            }
        }

        Ok(crate::private::ConfigAndSpan {
            config,
            span: config_content_and_span.span(),
        })
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
    fn load_file(span: Span, file_relative_path: impl AsRef<str>) -> MacroResult<String> {
        let file_relative_path = file_relative_path.as_ref();

        let file_full_path = {
            let invoker_file_path = some_or_fail! {
                span,
                span.local_file(),
                    "Rust source file that invoked readme_code_extractor_lib::load_file(...) \
                    (through one of readme_code_extractor's macros like all, all_by_file, nth, \
                    nth_by_file) for file with relative path {file_relative_path} \
                    should have a known location."
            };
            let invoker_parent_dir = some_or_fail!(
                span,
                invoker_file_path.parent(),
                "Rust source file that invoked readme_code_extractor_lib::load_file(...) \
                    (through one of readme_code_extractor's macros like all, all_by_file, nth, \
                    nth_by_file) for file with relative path {file_relative_path} \
                    may exist, but we can't get its parent directory."
            );
            invoker_parent_dir.join(file_relative_path)
        };

        // Error handling is modelling https://doc.rust-lang.org/nightly/src/core/result.rs.html
        // > `fn unwrap_failed`, which invokes `panic!("{msg}: {error:?}");`
        let content = ok_or_fail!(
            span,
            std::fs::read_to_string(&file_full_path),
            "Expecting a file {}, but opening it failed: {:?}",
            file_full_path
                .to_str()
                .unwrap_or("(PATH UNKNOWN OR NOT UTF-8)")
        );
        Ok(content)
    }

    #[doc(hidden)]
    pub fn readme_load(config_and_span: &impl ConfigAndSpan) -> MacroResult<impl ReadmeLoaded> {
        let markdown_file_local_path = config_and_span.config().markdown_file_local_path();
        let span = config_and_span.span();
        let markdown_file_content = load_file(span, markdown_file_local_path)?;
        Ok(crate::private::ReadmeLoaded {
            //span,
            markdown_file_local_path,
            markdown_file_content,
            config: config_and_span.config(),
        })
    }

    #[doc(hidden)]
    pub fn readme_extract<'a>(
        load: &'a impl crate::public::ReadmeLoaded,
    ) -> MacroDeepResult<impl crate::public::ReadmeExtracted<'a>> {
        let mut all_blocks = crate::public::ReadmeBlocksIter::new(
            /*load.span(),*/ load.markdown_file_content(),
        )
        .peekable();

        let (preamble_text, preamble_code) = if load.config().preamble().is_none() {
            (None, None)
        } else {
            let preamble_text = if let Some(block) = all_blocks.peek() {
                if matches!(block, &Ok(crate::private::ReadmeBlock::Text(_)) | &Err(_)) {
                    Some(all_blocks.next().unwrap()?) // .unwrap() is ok, because we've just peeked.
                } else {
                    None
                }
            } else {
                None
            };
            let preamble_code = if let Some(block) = all_blocks.peek() {
                if matches!(block, &Ok(crate::private::ReadmeBlock::Code(_)) | &Err(_)) {
                    Some(all_blocks.next().unwrap()?) // .unwrap() is ok, because we've just peeked.
                } else {
                    None
                }
            } else {
                None
            };
            (preamble_text, preamble_code)
        };

        //let source_file_full_path = load.source_file_full_path();
        Ok(crate::private::ReadmeExtracted {
            //span: load.span(),
            markdown_file_local_path: load.markdown_file_local_path(),
            preamble_text,
            preamble_code,
            non_preamble_blocks: all_blocks,
        })
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
            pub struct Tags<'a> {
                /// A list of strings to be injected after the injected
                /// [crate::private::config::Headers::prefix_before_tag], and before the
                /// beginning of the existing code of each non-preamble code block.
                ///
                /// Each string from this list is to be used exactly once, one per each non-preamble
                /// code block. The number of strings in this list has to be the same as the number
                /// of non-preamble code blocks.
                ///
                /// Example of useful tags: Names of test functions (or parts of such names) to
                /// generate, one per each non-preamble code block.
                pub tags: Vec<&'a str>,

                /// Content to be injected at the beginning of each non-preamble code block, but
                /// AFTER a tag.
                ///
                /// Example of useful content of a tag when generating test functions: `() {`.
                pub after_tag: &'a str,
            }
        }

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(default)]
        pub struct Headers<'a> {
            /// Prefix to be injected at the beginning of any non-preamble code block, even before
            /// an tag (if any).
            ///
            /// Example of useful prefix: `#[test] fn test_` for test functions to generate.
            pub prefix_before_tag: &'a str,

            #[serde(borrow)]
            pub tags: Option<headers::Tags<'a>>,
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(default)]
    #[non_exhaustive]
    pub struct Config<'a> {
        /// **Relative** path (relative to the directory of Rust source file that invoked the chain
        /// of macros). Defaults to "README.md".
        #[serde(default = "default_markdown_file_local_path")]
        pub markdown_file_local_path: &'a str,
        /// @TODO Document (here, and in TOML examples) that prefix_before_preamble
        /// CAN be set & used even IF preamble is set to [config::Preamble::None].
        pub start_prefix: &'a str,

        #[serde(borrow)]
        pub preamble: config::Preamble<'a>,

        #[serde(borrow, default = "default_config_ordinary_code_headers")]
        pub ordinary_code_headers: Option<config::Headers<'a>>,

        /// Suffix to be appended at the end of any non-preamble code block.
        ///
        /// Example of a useful suffix for generating test functions: `}`.
        pub ordinary_code_suffix: &'a str,

        pub final_suffix: &'a str,
    }
    pub fn default_markdown_file_local_path() -> &'static str {
        "README.md"
    }
    pub fn default_config_ordinary_code_headers<'a>() -> Option<config::Headers<'a>> {
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
        pub span: Span,
    }

    #[derive(Debug)]
    pub struct ReadmeLoaded<'a> {
        //pub span: &'a Span,
        pub markdown_file_content: String,
        pub markdown_file_local_path: &'a str,
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
        //pub span: &'a Span,
        pub markdown_file_local_path: &'a str,

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

    impl<'a> Trait for crate::private::config::headers::Tags<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> Default for crate::private::config::headers::Tags<'a> {
        fn default() -> Self {
            Self {
                tags: vec![],
                after_tag: "",
            }
        }
    }
    impl<'a> crate::public::config::headers::Tags for crate::private::config::headers::Tags<'a> {
        fn tags(&self) -> &[&str] {
            &self.tags
        }
        fn after_tag(&self) -> &str {
            &self.after_tag
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
                prefix_before_tag: "",
                tags: None,
            }
        }
    }

    impl<'a> crate::public::config::Headers for crate::private::config::Headers<'a> {
        fn prefix_before_tag(&self) -> &str {
            &self.prefix_before_tag
        }
        fn tags(&self) -> Option<&dyn crate::public::config::headers::Tags> {
            if let Some(tags) = &self.tags {
                Some(tags)
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
                markdown_file_local_path: crate::private::default_markdown_file_local_path(),

                start_prefix: "",
                preamble: crate::private::config::Preamble::None,

                ordinary_code_headers: crate::private::default_config_ordinary_code_headers(),
                ordinary_code_suffix: "",

                final_suffix: "",
            }
        }
    }
    impl<'a> crate::public::Config for crate::private::Config<'a> {
        fn markdown_file_local_path(&self) -> &str {
            self.markdown_file_local_path
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
        fn span(&self) -> Span {
            self.span
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
        fn span(&self) -> Span {
            self.span
        }
    }

    impl<'a> Trait for crate::private::ReadmeLoaded<'a> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &TraitParam) {}
    }
    impl<'a> crate::public::ReadmeLoaded for crate::private::ReadmeLoaded<'a> {
        /*fn span(&self) -> &Span {
            self.span
        }*/
        fn markdown_file_local_path(&self) -> &str {
            self.markdown_file_local_path
        }
        fn markdown_file_content(&self) -> &str {
            &self.markdown_file_content
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
        /*fn span(&self) -> &Span {
            self.span
        }*/
        fn markdown_file_local_path(&self) -> &str {
            self.markdown_file_local_path
        }
        fn preamble_text(&self) -> Option<&dyn crate::public::ReadmeBlock> {
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
            crate::public::string_literal_content(&literal)
                .unwrap()
                .as_ref(),
            content
        );
    }

    #[test]
    fn string_literal_from_str_ordinary() {
        let content = "ordinary literal";

        let enclosed = format!("\"{content}\"");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(
            crate::public::string_literal_content(&literal)
                .unwrap()
                .as_ref(),
            content
        );
    }

    #[test]
    fn string_literal_from_str_raw_0() {
        let content = "ordinary literal";

        let enclosed = format!("r\"{content}\"");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(
            crate::public::string_literal_content(&literal)
                .unwrap()
                .as_ref(),
            content
        );
    }

    #[test]
    fn string_literal_from_str_raw_1() {
        let content = "ordinary literal";

        let enclosed = format!("r#\"{content}\"#");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(
            crate::public::string_literal_content(&literal)
                .unwrap()
                .as_ref(),
            content
        );
    }

    #[test]
    fn string_literal_from_str_raw_2() {
        let content = "ordinary literal";

        let enclosed = format!("r##\"{content}\"##");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(
            crate::public::string_literal_content(&literal)
                .unwrap()
                .as_ref(),
            content
        );
    }
}
