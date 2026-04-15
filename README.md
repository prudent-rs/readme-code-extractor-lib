# readme-code-extractor-lib

Internal: `readme-code-extractor-lib` is shared between `readme-code-extractor` and
`readme-code-extractor-proc`. Do not use directly. Only use through `readme-code-extractor`'s
macros.

The only reason for this crate to exist as separate from readme-code-extractor is documentation on
docs.rs.

## Stability

This crate doesn't necessarily follow semver. All public types and traits are sealed. Outside this
crate its:

- Traits are sealed.
- Types are "sealed":
  - can't be used/referred to - other than with a generic parameter which implements a sealed trait
  - can't be instantiated (other than with `Default::default()`)
  - instances can't be cloned.
- Values
  - are returned only as immutable and with an opaque type: `Box<dyn
    readme_code_extractor_lib::traits::Config>`.
  - value parts are returned by immutable (shared) references, and with a non-Boxed opaque type,
    like `&dyn readme_code_extractor_lib::traits::config::Headers`.

We may have new fields added, and as far as `Default` value(s) are valid/good, there's no need for a
new major version.

## TOML only

We use only TOML deserialization with [`toml-rs/toml`](https://github.com/toml-rs). No other formats
(JSON, [`eternal-io/keon`](https://github.com/eternal-io/keon),
[`ron-rs/ron`](https://github.com/ron-rs/ron)... ). Why? Because TOML is

- simple and readable
- used by Rust community already
- both clean and expressive enough for simple Rust values, see `toml-rs/toml` ->
  - [`crates/toml/examples/enum_external.rs`](https://github.com/toml-rs/toml/blob/main/crates/toml/examples/enum_external.rs)
  - [`crates/toml/tests/serde/de_enum.rs`](https://github.com/toml-rs/toml/blob/main/crates/toml/tests/serde/de_enum.rs)
    -> `fn value_from_str()`
- syntax highlighted by ["Extended **Embedded**
  Languages"](https://marketplace.visualstudio.com/items?itemName=ruschaaf.extended-embedded-languages)
  in VS Code. That also works **in raw strings** passed to `#![doc = r#"..."#]` or `#[doc =
  r#"..."#]` (and other attributes).
