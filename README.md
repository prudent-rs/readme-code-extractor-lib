# mce-lib Markdown Code Extractor library (part of mce)

Mostly internal: Most of `mce-lib` is to be used by
[`crates.io/crates/mce`](https://crates.io/crates/mce)
([prudent-rs/mce](https://github.com/prudent-rs/mce)) and `mce-proc`
[`crates.io/crates/mce-proc`](https://crates.io/crates/mce-proc)
([`prudent-rs/mce-proc`](https://github.com/prudent-rs/mce-proc)) only, not directly.

The initial reason for this crate to exist as separate from `mce` was to have examples and
up-to-date published documentation (on docs.rs). As a benefit, `mce-lib` serves for documentation of
configuration fields of `mce` (and also `mce-proc`).

## Configuration in TOML only

See [`prudent/mce` -> `README.md`](https://github.com/prudent-rs/mce/blob/main/README.md)
for why configuration is in TOML only.
