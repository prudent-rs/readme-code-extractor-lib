#![doc = include_str!("../README.md")]

use core::error::Error;
use serde::{Deserialize, Serialize};

// Variations and params
// - preamble prefix
// - each item's prefix and suffix
// - list of idents (or their parts) to use with each non-preamble item

pub mod config {
    use serde::{Deserialize, Serialize};

    ///
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub enum Preamble {
        /// No preamble - the very first code block is a non-Preamble block (handled by injecting
        /// any header and/or body strings if set in [crate::Config]).
        NoPreamble,
        /// Expecting a preamble, but no special handling - pass as-is. Any [Headers] and/or
        /// [crate::Config::item_body_suffix] will NOT be injected.
        CopyVerbatim,
        /// Expecting the very first code block to contain `item`s only (as per
        /// [`item`](https://lukaswirth.dev/tlborm/decl-macros/minutiae/fragment-specifiers.html#item)
        /// captured by declarative macros (ones defined with `macro_rules!`)). For example,
        /// `struct` definitions, `use` or `pub use` imports.
        ///
        /// The [String] value is a prefix injected before each item (in the same preamble, that is,
        /// the first code block). Example of a potentially useful prefix:
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
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Clone, Debug)]
        #[serde(default)]
        pub struct Inserts {
            /// A list of strings to be injected after the injected
            /// [crate::config::Headers::prefix_before_insert], and before the beginning of the
            /// existing code of any non-preamble code block. Each string from this list is to be
            /// used exactly once, one per non-preamble code block. The number of strings in this
            /// list has to be the same as the number of non-preamble code blocks.
            ///
            /// Example of useful inserts: Names of test functions to generate, one per each
            /// non-preamble code block.
            pub inserts: Vec<String>,

            /// Content to be injected at the beginning of any non-preamble code block, but AFTER an
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

    #[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    // **Relative** path (relative to the directory of Rust source file that invoked the chain of
    // macros). Defaults to "README.md".
    pub file_path: String,

    /// NOT a part of public API. Set by crate `readme-code-extractor`.
    pub invoker_file_path: Option<String>,

    pub preamble: config::Preamble,

    pub headers: Option<config::Headers>,

    /// Suffix to be appended at the end of any non-preamble code block.
    ///
    /// Example of useful inserts for generating test functions: `}`.
    pub item_body_suffix: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            file_path: "README.md".to_owned(),
            invoker_file_path: None,

            preamble: config::Preamble::NoPreamble,

            headers: None,
            item_body_suffix: "".to_owned(),
        }
    }
}

type ValidationResult = Result<(), Box<dyn Error>>;

impl Config {
    /// Check integrity. However, it doesn't check whether [Config::file_path] points at a real
    /// file. And it doesn't check that [Config::invoker_directory] is `Some(...)`.
    pub fn validate_ready_except_invoker_directory(&self) -> ValidationResult {
        Ok(())
    }
    /// Like validate_ready_except_invoker_directory, plus check that [Config::invoker_directory] is
    /// `Some(...)`.
    pub fn validate_ready(&self) -> ValidationResult {
        Ok(())
    }
}
