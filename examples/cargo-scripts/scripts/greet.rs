#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2021"
---

fn main() {
    println!("Hello from a Rust cargo-script (RFC 3502)!");
}
