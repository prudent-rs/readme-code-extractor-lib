#![doc = include_str!("../README.md")]

extern crate alloc;
use alloc::string::String;
//use core::str::FromStr;
use proc_macro2::Literal;

mod string_literal_content {
    use proc_macro2::Literal;

    struct OwnedStringSlice {
        s: String,
        start_incl: usize,
        end_excl: usize,
    }
    impl OwnedStringSlice {
        pub fn new(s: String, start_incl: usize, end_excl: usize) -> Self {
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
    pub fn string_literal_content(literal: &Literal) -> impl AsRef<str> {
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
}
pub use string_literal_content::string_literal_content;

/// Restriction: We support only files that are in UTF-8 (the content is in UTF-8).
///
/// Return content of the file.
///
/// This function is NOT testable here, because it requires a literal that has [proc_macro2::Span]
/// (as returned by [proc_macro2::Literal::span]) that comes from a real file and not from a test.
/// (That is, [proc_macro2::Span::local_file] must return [Some].)
///
/// Therefore, this function is tested as a part of `prudent-rs/readme_code_extractor_proc`.
pub fn load_file(file_relative_path: &Literal) -> String {
    let span = file_relative_path.span();

    let file_relative_path = string_literal_content(file_relative_path);
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
pub mod misc {
    /// Intentionally NOT public.
    pub(crate) struct SealedTraitParam {}
    pub trait SealedTrait {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam);
    }

    /// Intentionally NOT public.
    pub(crate) struct SealedTraitImpl {}
    impl SealedTrait for SealedTraitImpl {
        fn _seal(&self, _: &SealedTraitParam) {}
    }
}

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

/// Internal/Only for prudent-rs/readme-code-extractor. SemVer-exempt!
pub mod traits {
    use proc_macro2::Span;
    pub mod config {
        //use alloc::string::String;

        pub trait Preamble: crate::misc::SealedTrait {
            fn is_no_preamble(&self) -> bool;

            fn is_copy_verbatim(&self) -> bool;

            /// If [None], then the preamble is NOT
            /// [crate::types::config::Preamble::ItemsWithPrefix]. If [Some], then the preamble IS
            /// [crate::types::config::Preamble::ItemsWithPrefix], regardless of whether the
            /// &[`str`] is empty or not. If &[`str`] is empty, then it's the same as if
            /// [Preamble::is_copy_verbatim] was `true`.
            fn is_items_with_prefix(&self) -> Option<&str>;
        }

        pub mod headers {
            use alloc::string::String;

            pub trait Inserts: crate::misc::SealedTrait {
                // - NOT returning an [Iterator], because [Iterator] would need to be `Box`-ed as
                //   `Box<&dyn Iterator<Item = &'a str>>`. Or we would need to export a custom
                //   Iterator type.
                // - NOT returning `impl Iterator<Item = &'a str>`, because then this trait would
                //   NOT be dyn-compatible.
                // - A slice is more flexible/useable than an [Iterator].
                fn inserts<'a>(&'a self) -> &'a [String];

                fn after_insert(&self) -> &str;
            }
        }

        pub trait Headers: crate::misc::SealedTrait {
            fn prefix_before_insert(&self) -> &str;
            fn inserts(&self) -> Option<&dyn headers::Inserts>;
        }
    }

    pub trait Config: crate::misc::SealedTrait {
        fn file_path(&self) -> &str;

        fn preamble(&self) -> &dyn config::Preamble;

        fn ordinary_code_headers(&self) -> Option<&dyn config::Headers>;

        fn ordinary_code_suffix(&self) -> &str;
    }

    pub trait ConfigAndSpan: crate::misc::SealedTrait {
        fn config(&self) -> &dyn Config;
        fn span(&self) -> &Span;
    }
}

// @TODO conditional compilation - for docs.rs only. See prudent
//
//pub use priv_types as types;

/// Internal/Only for prudent-rs/readme-code-extractor. SemVer-exempt!
///
/// Public only when on docs.rs, so they get documented. Feature that enables them to be public
/// fails with a compile error if used outside of docs.rs.
pub mod priv_types {
    use alloc::string::String;
    use core::marker::PhantomData;
    use proc_macro2::Span;
    use serde::{Deserialize, Serialize};

    pub mod config {
        use alloc::string::String;
        use serde::{Deserialize, Serialize};

        /// Whether the very first code block is a preamble that needs special handling.
        ///
        /// Intentionally NOT implementing [Clone], as we don't want user code to make copies.
        #[derive(Serialize, Deserialize, Debug)]
        pub enum Preamble {
            /// No preamble - the very first code block is a non-Preamble block (handled by
            /// injecting any header and/or body strings if set in [crate::types::Config]).
            NoPreamble,

            /// Expecting a preamble, but no special handling - pass as-is. Any [Headers] and/or
            /// [crate::types::Config::ordinary_code_suffix] will NOT be applied
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
            ItemsWithPrefix(String),
        }

        pub mod headers {
            use alloc::{string::String, vec::Vec};
            use serde::{Deserialize, Serialize};

            #[derive(Serialize, Deserialize, Debug)]
            #[serde(default)]
            pub struct Inserts {
                /// A list of strings to be injected after the injected
                /// [crate::types::config::Headers::prefix_before_insert], and before the beginning
                /// of the existing code of each non-preamble code block. Each string from this list
                /// is to be used exactly once, one per each non-preamble code block. The number of
                /// strings in this list has to be the same as the number of non-preamble code
                /// blocks.
                ///
                /// Example of useful inserts: Names of test functions (or parts of such names) to
                /// generate, one per each non-preamble code block.
                pub(crate) inserts: Vec<String>,

                /// Content to be injected at the beginning of each non-preamble code block, but
                /// AFTER an insert.
                ///
                /// Example of useful inserts for generating test functions: `() {`.
                pub(crate) after_insert: String,
            }
        }

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(default)]
        pub struct Headers {
            /// Prefix to be injected at the beginning of any non-preamble code block, even before
            /// an insert (if any).
            ///
            /// Example of useful prefix: `#[test] fn test_` for test functions to generate.
            pub(crate) prefix_before_insert: String,

            pub(crate) inserts: Option<headers::Inserts>,
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(default)]
    #[non_exhaustive]
    pub struct Config {
        /// **Relative** path (relative to the directory of Rust source file that invoked the chain
        /// of macros). Defaults to "README.md".
        pub(crate) file_path: String,

        pub(crate) preamble: config::Preamble,

        pub(crate) ordinary_code_headers: Option<config::Headers>,

        /// Suffix to be appended at the end of any non-preamble code block.
        ///
        /// Example of useful inserts for generating test functions: `}`.
        pub(crate) ordinary_code_suffix: String,
    }

    /// NO need to derive Serialize, Deserialize or Default.
    #[non_exhaustive]
    pub struct ConfigAndSpan {
        pub(crate) config: Config,
        pub(crate) span: Span,
    }

    pub struct Read {
        source_file_content: String,
        //config_and_span: ConfigAndSpan
    }

    pub(crate) struct Extracted<'a> {
        pub(crate) preamble: Option<&'a str>,
    }
}

mod trait_impls {
    use crate::misc::{SealedTrait, SealedTraitParam};
    use alloc::string::String;
    use core::marker::PhantomData;

    impl SealedTrait for crate::priv_types::config::Preamble {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam) {}
    }
    impl Default for crate::priv_types::config::Preamble {
        fn default() -> Self {
            Self::NoPreamble
        }
    }
    impl crate::traits::config::Preamble for crate::priv_types::config::Preamble {
        fn is_no_preamble(&self) -> bool {
            matches!(self, Self::NoPreamble)
        }
        fn is_copy_verbatim(&self) -> bool {
            matches!(self, Self::CopyVerbatim)
        }
        fn is_items_with_prefix(&self) -> Option<&str> {
            if let Self::ItemsWithPrefix(s) = self {
                Some(s)
            } else {
                None
            }
        }
    }

    impl SealedTrait for crate::priv_types::config::headers::Inserts {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam) {}
    }
    impl Default for crate::priv_types::config::headers::Inserts {
        fn default() -> Self {
            Self {
                inserts: vec![],
                after_insert: "".to_owned(),
            }
        }
    }
    impl crate::traits::config::headers::Inserts for crate::priv_types::config::headers::Inserts {
        fn inserts<'a>(&'a self) -> &'a [String] {
            &self.inserts
        }
        fn after_insert(&self) -> &str {
            &self.after_insert
        }
    }

    impl SealedTrait for crate::priv_types::config::Headers {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam) {}
    }
    impl Default for crate::priv_types::config::Headers {
        fn default() -> Self {
            Self {
                prefix_before_insert: "".to_owned(),
                inserts: None,
            }
        }
    }

    impl crate::traits::config::Headers for crate::priv_types::config::Headers {
        fn prefix_before_insert(&self) -> &str {
            &self.prefix_before_insert
        }
        fn inserts(&self) -> Option<&dyn crate::traits::config::headers::Inserts> {
            if let Some(inserts) = &self.inserts {
                Some(inserts)
            } else {
                None
            }
        }
    }

    impl SealedTrait for crate::priv_types::Config {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam) {}
    }
    impl Default for crate::priv_types::Config {
        fn default() -> Self {
            Self {
                file_path: "README.md".to_owned(),

                preamble: crate::priv_types::config::Preamble::NoPreamble,

                ordinary_code_headers: None,
                ordinary_code_suffix: "".to_owned(),
            }
        }
    }
    impl crate::traits::Config for crate::priv_types::Config {
        fn file_path(&self) -> &str {
            &self.file_path
        }
        fn preamble(&self) -> &dyn crate::traits::config::Preamble {
            &self.preamble
        }
        fn ordinary_code_headers(&self) -> Option<&dyn crate::traits::config::Headers> {
            if let Some(headers) = &self.ordinary_code_headers {
                Some(headers)
            } else {
                None
            }
        }
        fn ordinary_code_suffix(&self) -> &str {
            &self.ordinary_code_suffix
        }
    }

    impl SealedTrait for crate::priv_types::ConfigAndSpan {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam) {}
    }
    impl crate::traits::ConfigAndSpan for crate::priv_types::ConfigAndSpan {
        fn config(&self) -> &dyn crate::traits::Config {
            &self.config
        }
        fn span(&self) -> &proc_macro2::Span {
            &self.span
        }
    }
}

fn config_and_span(config_content_literal: &Literal) -> impl traits::ConfigAndSpan {
    let config_content = crate::string_literal_content(config_content_literal);
    let config_content = config_content.as_ref();
    let config = toml::from_str::<priv_types::Config>(config_content);

    match config {
        Ok(config) => priv_types::ConfigAndSpan {
            config,
            span: config_content_literal.span(),
        },
        Err(e) => {
            panic!(
                "Couldn't parse given literal's content as an expected TOML config. Content: {config_content}\n{e:?}"
            )
        }
    }
}

/// Internal, used between crates `readme-code-extractor-lib` and `readme-code-extractor-proc` and
/// `readme-code-extractor` to assure that they're of the same version.
#[doc(hidden)]
pub const fn is_exact_version(expected_version: &'static str) -> bool {
    matches!(expected_version.as_bytes(), b"0.1.0")
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
        assert_eq!(crate::string_literal_content(&literal).as_ref(), content);
    }

    #[test]
    fn string_literal_from_str_ordinary() {
        let content = "ordinary literal";

        let enclosed = format!("\"{content}\"");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(crate::string_literal_content(&literal).as_ref(), content);
    }

    #[test]
    fn string_literal_from_str_raw_0() {
        let content = "ordinary literal";

        let enclosed = format!("r\"{content}\"");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(crate::string_literal_content(&literal).as_ref(), content);
    }

    #[test]
    fn string_literal_from_str_raw_1() {
        let content = "ordinary literal";

        let enclosed = format!("r#\"{content}\"#");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(crate::string_literal_content(&literal).as_ref(), content);
    }

    #[test]
    fn string_literal_from_str_raw_2() {
        let content = "ordinary literal";

        let enclosed = format!("r##\"{content}\"##");
        let literal = Literal::from_str(&enclosed).unwrap();

        assert_eq!(crate::string_literal_content(&literal).as_ref(), content);
    }

    #[test]
    fn load_file_() {
        let literal = Literal::string("tests/file_1.txt");
        let file_content = crate::load_file(&literal);
        assert_eq!(file_content, "Hi from file_1.txt");
    }
}
