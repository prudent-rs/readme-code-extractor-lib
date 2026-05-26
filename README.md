# mce-lib Markdown Code Extractor library (part of mce)

Mostly internal: Most of `mce-lib` is to be used by
[`crates.io/crates/mce`](https://crates.io/crates/mce)
([prudent-rs/mce](https://github.com/prudent-rs/mce)) and `mce-proc`
[`crates.io/crates/mce-proc`](https://crates.io/crates/mce-proc)
([`prudent-rs/mce-proc`](https://github.com/prudent-rs/mce-proc)) only, not directly.

The only initial reason for this crate to exist as separate from `mce` was to have examples and
up-to-date published documentation (on docs.rs).

## Configuration in TOML only

We use only TOML deserialization with [`toml-rs/toml`](https://github.com/toml-rs) for
configuration. No other formats (JSON, [`eternal-io/keon`](https://github.com/eternal-io/keon),
[`ron-rs/ron`](https://github.com/ron-rs/ron)... ). Why? Because TOML is

- simple and readable
- used by Rust community already
- both clean and expressive enough for simple Rust values, see `toml-rs/toml` ->
  - [`crates/toml/examples/enum_external.rs`](https://github.com/toml-rs/toml/blob/main/crates/toml/examples/enum_external.rs)
  - [`crates/toml/tests/serde/de_enum.rs`](https://github.com/toml-rs/toml/blob/main/crates/toml/tests/serde/de_enum.rs)
    -> `fn value_from_str()`
- syntax highlighted by ["Extended **Embedded**
  Languages"](https://marketplace.visualstudio.com/items?itemName=ruschaaf.extended-embedded-languages)
  in VS Code. That also works **in raw strings** passed to
  - `#![doc = r#"..."#]` or
  - `#[doc  = r#"..."#]` (and other attributes).
