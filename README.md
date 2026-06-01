# OSPL

OSPL is an experimental programming language toolchain written in Rust. The
workspace contains a parser, compiler, bytecode/runtime type definitions, a VM,
and a small command-line tool named `ospl-toolchain`.

## Requirements

This repo is pinned to Rust `1.88.0` in `rust-toolchain.toml` because the locked
dependencies require at least that compiler version.

Install the toolchain if needed:

```sh
rustup toolchain install 1.88.0
```

## Run the Toolchain

From the repo root:

```sh
cargo run -p ospl-toolchain -- --help
```

Create a new OSPL package:

```sh
cargo run -p ospl-toolchain -- new hello -k binary
cd hello
```

Add a `main.ospl` file:

```ospl
def x = 1;
```

Build and run it:

```sh
cargo +1.88.0 run --manifest-path ../Cargo.toml -p ospl-toolchain -- scratch-run
```

The build output is written to `build/dist.ospb`. You can run bytecode directly
with:

```sh
cargo +1.88.0 run --manifest-path ../Cargo.toml -p ospl-toolchain -- exec build/dist.ospb
```

## Package Format

`package.yml` maps package targets to source files or folders. Binary packages
must declare a `binary` include:

```yaml
name: hello
version: 1.0.0

includes:
  binary: !File "main.ospl"

extensions: {}
```

Library packages use `library` instead of `binary`.

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
