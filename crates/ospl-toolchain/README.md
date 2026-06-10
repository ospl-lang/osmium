# ospl-toolchain

Command-line package builder and runner for OSPL.

Commands:

- `new <name> -k <binary|library>` creates a package folder with `package.kdl`.
- `build` compiles the current package into `build/dist.ospb`.
- `scratch-run` rebuilds the current package and immediately runs it.
- `exec <path>` runs an existing bytecode file.

Run from the workspace root:

```sh
cargo run -p ospl-toolchain -- --help
```

Run against an OSPL package outside the workspace:

```sh
cargo +1.88.0 run --manifest-path /path/to/ospl/Cargo.toml -p ospl-toolchain -- scratch-run
```
