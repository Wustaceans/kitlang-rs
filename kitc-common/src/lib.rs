//! Common types and utilities for the Kit compiler.
//!
//! This crate provides shared functionality between the Kit frontend (`kitlang`)
//! and the C FFI handling (`kitc-ffi`):
//!
//! - **Compiler detection and invocation** ([`compiler`]): Toolchain detection
//!   (GCC/Clang/MSVC), flag translation, and compiler command-line construction.
//! - **System compiler discovery** ([`compiler_detect`]): Finding the system C
//!   compiler, querying its built-in include paths, and bundling minimal
//!   builtin headers for preprocessing.
//! - **Error types** ([`error`]): Shared error types for compilation and I/O errors.

pub mod compiler;
pub mod compiler_detect;
pub mod error;

#[cfg(windows)]
pub mod msvc;

pub use compiler::{CompilerMeta, CompilerOptions, Toolchain};
pub use compiler_detect::{
    CompilerInfo, get_builtin_headers, get_compiler_info, get_system_include_dirs,
    init_compiler_info,
};
