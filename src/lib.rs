#![doc = include_str!("../README.md")]
#![no_std]
extern crate alloc;


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
            /// No preamble - the very first code block is a non-Preamble block (handled by injecting
            /// any header and/or body strings if set in [crate::Config]).
            NoPreamble,
            /// Expecting a preamble, but no special handling - pass as-is. Any [Headers] and/or
            /// [crate::Config::ordinary_code_suffix] will NOT be applied (prefixed/inserted).
            CopyVerbatim,
            /// Expecting the very first code block to contain `item`s ONLY (as per
            /// [`item`](https://lukaswirth.dev/tlborm/decl-macros/minutiae/fragment-specifiers.html#item)
            /// captured by declarative macros (ones defined with `macro_rules!`)). For example,
            /// `struct` definitions, `use` or `pub use` imports.
            ///
            /// The [String] value is a prefix injected before each item (located in the same preamble,
            /// that is, the very first code block). Example of a potentially useful prefix:
            /// - `#[allow(unused_imports)]`, or
            /// - `# #[allow(unused_imports)]` where the leading `#` makes that line
            ///   [hidden](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#hiding-portions-of-the-example)
            ///   in the generated documentation.
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
                /// [crate::config::Headers::prefix_before_insert], and before the beginning of the
                /// existing code of each non-preamble code block. Each string from this list is to be
                /// used exactly once, one per each non-preamble code block. The number of strings in
                /// this list has to be the same as the number of non-preamble code blocks.
                ///
                /// Example of useful inserts: Names of test functions (or parts of such names) to
                /// generate, one per each non-preamble code block.
                pub inserts: Vec<String>,

                /// Content to be injected at the beginning of each non-preamble code block, but AFTER an
                /// insert.
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
            /// Prefix to be injected at the beginning of any non-preamble code block, even before an
            /// insert (if any).
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
    pub struct Config<S: SealedTrait> {
        _seal: PhantomData<S>,

        // **Relative** path (relative to the directory of Rust source file that invoked the chain of
        // macros). Defaults to "README.md".
        pub file_path: String,

        pub preamble: config::Preamble,

        pub ordinary_code_headers: Option<config::Headers>,

        /// Suffix to be appended at the end of any non-preamble code block.
        ///
        /// Example of useful inserts for generating test functions: `}`.
        pub ordinary_code_suffix: String,
    }

    impl<S: SealedTrait> Default for Config<S> {
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

    /// Intentionally NOT public.
    struct SealedTraitParam {}
    pub trait SealedTrait {
        #[allow(private_interfaces)]
        fn _seal(_: &SealedTraitParam);
    }

    /// Intentionally NOT public.
    #[allow(dead_code)]
    struct SealedTraitImpl {}
    impl SealedTrait for SealedTraitImpl {
        fn _seal(_: &SealedTraitParam) {}
    }
}

/// Internal, used between crates `readme-code-extractor-core` and `readme-code-extractor` to
/// assure that they're of the same version.
#[doc(hidden)]
pub const fn is_exact_version(expected_version: &'static str) -> bool {
    matches!(expected_version.as_bytes(), b"0.1.0")
}

#[doc(hidden)]
const _ASSERT_VERSION: () = {
    if !crate::is_exact_version(env!("CARGO_PKG_VERSION")) {
        panic!("prudent-rs/readme-code-extractor-core is of different version than expected.");
    }
};
