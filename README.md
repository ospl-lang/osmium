# OSPL

OSPL is an experimental programming language toolchain written in Rust. The
workspace contains a parser, compiler, bytecode/runtime type definitions, a VM,
and a small command-line tool named `ospl-toolchain`.

## Learn OSPL

The Absolute Guide to OSPL is available [here](https://github.com/ospl-lang/book/blob/main/The%20Absolute%20Guide%20to%20OSPL.pdf).

The OSPL website can be found [here](https://ospl-lang.github.io)

## Requirements

This repo is pinned to Rust `1.88.0` in `rust-toolchain.toml` because the locked
dependencies require at least that compiler version.

Install the toolchain if needed:

```sh
rustup toolchain install 1.88.0
```

## Workspace Layout

- `crates/ospl-common`: shared AST, bytecode, runtime values, and frame types.
- `crates/ospl-parser`: lexer and parser for OSPL source.
- `crates/ospl-compiler`: compiler from parsed AST into VM instructions.
- `crates/ospl-vm`: bytecode interpreter and runtime support.
- `crates/ospl-toolchain`: command-line package builder and runner.
- `test suite`: loose OSPL examples and experiments.

## Useful Commands

```sh
cargo check
cargo test
cargo run -p ospl-toolchain -- --help
```
