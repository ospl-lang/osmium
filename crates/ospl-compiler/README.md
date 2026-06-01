# ospl-compiler

Compiler from OSPL AST into VM instructions.

This crate consumes parsed package modules and emits `ospl-common` bytecode
instructions for the VM. It also tracks scopes, types, captures, imports, and
compile-time errors.

Key modules:

- `src/package.rs`: package/module registry used during compilation.
- `src/impls/stmt.rs`: statement compilation.
- `src/impls/expr.rs`: expression compilation.
- `src/impls/ffi.rs`: foreign function integration.
