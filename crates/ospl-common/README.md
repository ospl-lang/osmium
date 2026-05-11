# ospl-common

Shared data model for the OSPL toolchain.

This crate defines the AST structures, runtime values, bytecode instruction
types, list representation, and runtime frame/address types used by the parser,
compiler, and VM.

Use this crate when adding language syntax, compiler output, or VM behavior that
needs a shared representation across multiple crates.
