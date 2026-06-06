# ospl-vm

Bytecode interpreter for OSPL programs.

The VM executes optimized instructions from `ospl-common`, manages runtime
frames, stores values in an arena, performs basic garbage-collection accounting,
and supports foreign function calls through dynamically loaded C extensions.

Key modules:

- `src/lib.rs`: VM state and instruction dispatch.
- `src/arena.rs`: runtime value arena.
- `src/gc.rs`: reference-count bookkeeping.
- `src/ffi`: foreign function loading and calls.
- `src/tests`: VM behavior tests.
