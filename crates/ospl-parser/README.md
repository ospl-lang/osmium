# ospl-parser

Lexer and parser for OSPL source files.

The lexer turns source text into tokens. The parser converts those tokens into
the AST types from `ospl-common`. Top-level files are parsed as semicolon-ended
statements; block expressions use `{ ... }`.

Key modules:

- `src/lexer`: token definitions and lexical scanning.
- `src/parse/stmt.rs`: statements and top-level file parsing.
- `src/parse/expr.rs`: expressions, calls, lvalues, casts, and operators.
- `src/parse/ffi.rs`: foreign function syntax.
