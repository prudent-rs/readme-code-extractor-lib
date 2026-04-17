#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;
use alloc::string::{String, ToString};
use proc_macro2::Literal;
use toml::de::Error as TomlError;

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

#[doc(hidden)]
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

    let (start_incl, end_excl) = if first == '"' {
        // ordinary "string literals"
        let last = chars
            .next_back()
            .unwrap_or_else(|| panic!("Can't parse the last character of: {enclosed}"));
        assert_eq!(
            last, '"',
            "Expecting the last character to be a closing quote '\"', but it's: '{last}'."
        );
        for c in chars {
            if c == '\\' {
                panic!(
                    "When passing in an ordinary enclosed string literal \"...\", do not use \
                        any escaping (backslash). To pass in special characters, use an \
                        (unescaped) raw string literal like r\"...\", r#\"...\"#..., r##\"...\"## \
                        (and so on)."
                )
            }
        }
        (1, enclosed.len() - 2)
    } else if first == 'r' {
        // raw string literals

        todo!()
    } else {
        panic!(
            r###"Expecting a string literal, which would be either \"...\", or r\"...\",
                    r#\"...\"#, r##"..."## (and so on). But received: {enclosed}"###
        )
    };

    OwnedStringSlice::new(enclosed, start_incl, end_excl)
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
    //#[allow(dead_code)]
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

fn _f() -> &'static str {
    /*json*/
    r#"
    {"a": "b", "c": [1, 2, 3]}
    "#
}

pub mod traits {
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
}

pub mod types {
    use alloc::{borrow::ToOwned, string::String};
    use core::marker::PhantomData;
    use serde::{Deserialize, Serialize};

    pub mod config {
        use alloc::{borrow::ToOwned, string::String};
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
        impl Default for Preamble {
            fn default() -> Self {
                Self::NoPreamble
            }
        }

        pub mod headers {
            use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
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
                pub inserts: Vec<String>,

                /// Content to be injected at the beginning of each non-preamble code block, but
                /// AFTER an insert.
                ///
                /// Example of useful inserts for generating test functions: `() {`.
                pub after_insert: String,
            }
            impl Default for Inserts {
                fn default() -> Self {
                    Self {
                        inserts: vec![],
                        after_insert: "".to_owned(),
                    }
                }
            }
        }

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(default)]
        pub struct Headers {
            /// Prefix to be injected at the beginning of any non-preamble code block, even before
            /// an insert (if any).
            ///
            /// Example of useful prefix: `#[test] fn test_` for test functions to generate.
            pub prefix_before_insert: String,

            pub inserts: Option<headers::Inserts>,
        }
        impl Default for Headers {
            fn default() -> Self {
                Self {
                    prefix_before_insert: "".to_owned(),
                    inserts: None,
                }
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(default)]
    /// To prevent the users on depending on pattern matching completeness etc.
    #[non_exhaustive]
    pub struct Config<S: crate::misc::SealedTrait> {
        _seal: PhantomData<S>,

        /// **Relative** path (relative to the directory of Rust source file that invoked the chain
        /// of macros). Defaults to "README.md".
        pub file_path: String,

        pub preamble: config::Preamble,

        pub ordinary_code_headers: Option<config::Headers>,

        /// Suffix to be appended at the end of any non-preamble code block.
        ///
        /// Example of useful inserts for generating test functions: `}`.
        pub ordinary_code_suffix: String,
    }

    impl<S: crate::misc::SealedTrait> Default for Config<S> {
        fn default() -> Self {
            Config {
                _seal: PhantomData,

                file_path: "README.md".to_owned(),

                preamble: config::Preamble::NoPreamble,

                ordinary_code_headers: None,
                ordinary_code_suffix: "".to_owned(),
            }
        }
    }
}

mod trait_impls {
    use crate::misc::{SealedTrait, SealedTraitParam};
    use alloc::string::String;

    impl SealedTrait for crate::types::config::Preamble {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam) {}
    }
    impl crate::traits::config::Preamble for crate::types::config::Preamble {
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

    impl SealedTrait for crate::types::config::headers::Inserts {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam) {}
    }
    impl crate::traits::config::headers::Inserts for crate::types::config::headers::Inserts {
        fn inserts<'a>(&'a self) -> &'a [String] {
            &self.inserts
        }
        fn after_insert(&self) -> &str {
            &self.after_insert
        }
    }

    impl SealedTrait for crate::types::config::Headers {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam) {}
    }
    impl crate::traits::config::Headers for crate::types::config::Headers {
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

    impl<S: SealedTrait> SealedTrait for crate::types::Config<S> {
        #[allow(private_interfaces)]
        fn _seal(&self, _: &SealedTraitParam) {}
    }
    impl<S: SealedTrait> crate::traits::Config for crate::types::Config<S> {
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
}

pub fn from_toml(input: &str) -> Result<impl traits::Config, TomlError> {
    let cfg: types::Config<misc::SealedTraitImpl> = toml::from_str(input)?;
    Ok(cfg)
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
