## Ve for Rust

This is a port of the ruby gem https://github.com/Kimtaro/ve to Rust. 🦀

The Rust version is meant to be used with https://github.com/daac-tools/vibrato/, a great and blazingly fast mecab-compatible tokenizer, and an IPADIC dictionary which can be found in the same repo (under Releases).

(Support for other tokenizer crates like Lindera planned as well, should be easy)

I'm trying to mostly stay close to the original codebase, and use Rust ways and idioms where applicable.

In the future I'm planning to add tests comparing outputs from the ruby and rust version, making sure there's no unexpected differences in logic.

## Getting started
You can play around with the library as-is simply by cloning the repo and `cargo run`ning it. This will tokenize an example string using vibrato and then postprocess the tokens to return a more meaningful array of words.

The example code also shows in a simple way how to use this crate in your own application, provided that you're working with vibrato for tokenization.
