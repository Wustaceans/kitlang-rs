# Roadmap

This document outlines the planned development of the Rust-based Kit compiler.
The project is currently in the **bootstrapping phase**, focusing on a minimal but correct
compiler pipeline.

The roadmap is organized into *phases*, not strict deadlines.

## Phase 0: Core Compiler Infrastructure (current)

Goal: compile a small but valid subset of Kit to C99 and produce a working binary.

### Parsing & Representation

- [X] Grammar definition closely aligned with the original Haskell AST
- [X] Stable AST representation in Rust

### Code Generation (C backend)

Goal: lower a well-defined subset of Kit ("Kit Core") to portable C99.

#### Supported language features (Kit Core)

- [X] Top-level functions
- [X] Local variables (`var`)
- [X] Primitive types:
  - [X] `Int`
  - [X] `Bool`
  - [X] `CString`
  - [X] `Char` / `Float` literals
- [X] `if` expressions / statements
- [X] `while` and `for` loops
  - [X] Basic implementation
  - [X] `for i in X...Y` range loops
  - [X] `for i in array` (C array iteration)
  - [ ] `for x in iterable` via `kit.iterator` (iterator-based loops) — *planned*
- [X] Function calls (named, indirect, and qualified cross-module)
- [X] `return` / `break` / `continue`
- [X] `defer` statements (scope-exit cleanup, LIFO ordering)
- [X] `include` statements for C headers
- [X] Interoperability with C functions (e.g. `printf`)
- [X] `typedef` declarations
- [X] `struct` definitions, field access, initialization, const fields
- [X] `enum` definitions (incl. tagged unions), field access, defaults
- [X] Top-level and module-level `var` declarations (globals)
- [X] Array types and array literals
- [X] Multi-module programs:
  - [X] Single imports
  - [X] Wildcard (`.*`) imports
  - [X] Double-wildcard (`.**`) imports
  - [X] Qualified calls across modules
  - [X] Prelude resolution per module path

#### Parsed but not yet code-generated

These constructs are recognized by the parser but are currently dropped before
code generation (see `codegen/transpile/header.rs`, which zeroes them out). They
are tracked here so contributors know the gap between "parses" and "compiles".

- [ ] Traits (`trait_def`) — parsed, body discarded
- [ ] Trait implementations (`trait_impl`) — parsed, placeholder only
- [ ] Term rewriting / rulesets (`rule` / `rules`) — parsed, never applied
- [ ] `using` statements (rulesets / implicits) — parsed, never consumed
- [ ] Generics (type parameters) — syntax parsed and then skipped

#### Explicitly unsupported (for now)

These are features of the original Haskell compiler that are **not** on the
near-term plan:

- [ ] Generics / monomorphization
- [ ] Term rewriting (rulesets)
- [ ] Pattern matching (`match`)
- [ ] Implicits
- [ ] Traits / vtables

#### Backend behavior

- [X] Generate valid, readable C99 source code
- [X] Invoke the system C compiler to produce a binary
  - [X] Configurable compiler flags and toolchain
  - [X] Configurable library search paths
  - [X] Overridable C compiler binary (`--cc`)
- [X] Remove intermediate C files after successful compilation

### Error Handling

- [X] Replace panics with meaningful compiler errors
- [X] Add error location information (line and column)
- [X] Show a source code snippet for errors
- [X] Structured internal parser diagnostics (`codegen/parser/diagnostics.rs`)

### Testing

- [X] Unit testing using examples in `examples/`

## Phase 1: Compiler CLI & Developer Experience

Goal: improve usability and feedback during compilation.

- [X] Display elapsed compilation time
- [ ] Show compilation progress (progress bar or structured stages)

## Phase 2: Standard Library

Goal: provide a minimal but practical standard library.

- [ ] Use the original Kit stdlib as a starting point, with license and authorship disclaimers
- [ ] Provide C interoperability where needed
- [ ] Aim for zero-cost abstractions where possible
- [ ] Provide a `kit.iterator` module so `for x in iterable` works

## Phase 3: Package Manager & Project Workflow

Goal: make Kit projects ergonomic to build and manage.

- [ ] Introduce a Cargo-like workflow for Kit projects
- [ ] Support `kit.yaml` project manifests
- [ ] Documentation generation (inspired by `cargo doc`)

More information at <https://docs.kitlang.dev/package-management/>

## Phase 4: Documentation & Project Infrastructure

Goal: support users and contributors beyond the compiler itself.

- [X] Provide a landing page with examples and documentation (kitlang.dev)
- [ ] Compiler-specific documentation (architecture, internals, design decisions; docs.kitlang.dev)
- [ ] Blog / news section for project updates (optional, future)
- [ ] Set a Kit Style Guide based on stdlib code and Kit repos (optional, future). More information on this at <https://docs.kitlang.dev/style-guide/>

## Completed Work

These features are already implemented:

- Add linking flags for compatibility with C libraries
- Replace compiler panics with structured error messages
- Unit testing via example-based tests
- Display elapsed compilation time during compilation
- Structs, enums, typedefs, arrays, ranges, and multi-module imports
- Error location reporting with source snippets
- Configurable C compiler flags, library paths, and compiler binary
