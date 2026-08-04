//! MSVC (Microsoft Visual C++) toolchain support.
//!
//! This module centralises all Windows/MSVC-specific logic:
//!
//! - compiler discovery;
//! - build-environment capture;
//! - include-path discovery;
//! - flag translation for `cl.exe`.
//!
//! All items are only compiled on Windows (`#[cfg(windows)]` on the module declaration in `lib.rs`).

use find_msvc_tools::Tool as MsvcTool;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// --- Discovery (delegated to find-msvc-tools) ---

/// Cached result of discovery, populated once at first use.
static MSVC_TOOL: OnceLock<Option<MsvcTool>> = OnceLock::new();

fn get_tool() -> Option<&'static MsvcTool> {
    MSVC_TOOL
        .get_or_init(|| find_msvc_tools::find_tool(host_arch(), "cl.exe"))
        .as_ref()
}

/// Return the host architecture string expected by `find-msvc-tools`.
const fn host_arch() -> &'static str {
    env::consts::ARCH
}

/// Locate the MSVC `cl.exe` compiler.
pub fn find_msvc() -> Option<PathBuf> {
    get_tool().map(|t| t.path().to_path_buf())
}

/// Capture the environment required to drive `cl.exe`.
///
/// Returns a map of the environment variables that the MSVC toolchain requires
/// (INCLUDE, LIB, PATH, ...), as determined by `find-msvc-tools`.
pub fn msvc_environment() -> HashMap<String, String> {
    let original_env: HashMap<String, String> = env::vars().collect();

    get_tool()
        .map(|tool| {
            tool.env()
                .into_iter()
                .filter_map(|(k, v)| {
                    let k = k.to_string_lossy().to_string();
                    let v = v.to_string_lossy().to_string();
                    (!k.is_empty() && original_env.get(&k).map(String::as_str) != Some(v.as_str()))
                        .then_some((k, v))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// System include directories for MSVC, discovered from the toolchain's `INCLUDE` environment variable.
pub fn get_includes() -> Vec<PathBuf> {
    get_tool()
        .and_then(|tool| {
            tool.env()
                .into_iter()
                .find(|(k, _)| k.to_string_lossy().as_ref() == "INCLUDE")
                .map(|(_, v)| env::split_paths(v).collect())
        })
        .unwrap_or_default()
}

// --- Toolchain flag helpers ---

/// Returns `true` if `flag` is an MSVC standard-selection flag that should be stripped (the generated C
/// code must compile as C99, so the standard flag is re-applied last).
pub fn is_standard_flag(flag: &str) -> bool {
    flag.eq_ignore_ascii_case("/std:c11")
        || flag.eq_ignore_ascii_case("/std:c17")
        || flag.eq_ignore_ascii_case("/std:c99")
}

/// Compiler flags passed to `cl.exe` for C compilation.
pub fn compiler_flags() -> Vec<String> {
    vec!["/std:c11".to_string(), "/W4".to_string()]
}

/// C99/C11 standard flag for MSVC.
pub fn c99_standard_flag() -> Option<&'static str> {
    Some("/std:c11")
}

/// Output flag prefix for the executable (`/Fe` on MSVC).
pub const fn output_flag() -> &'static str {
    "/Fe"
}

/// Link library argument (e.g. `"m"` -> `"m.lib"`).
pub fn link_lib_arg(lib: &str) -> String {
    format!("{}.lib", lib)
}

/// Library search path argument (e.g. `path` -> `"/LIBPATH:path"`).
pub fn lib_path_arg(path: &Path) -> String {
    format!("/LIBPATH:{}", path.display())
}

/// Attached output argument for `cl.exe`.
///
/// MSVC's `cl.exe` requires the output flag and path as a single attached token
/// (`/Fe<exe>`). A detached `/Fe` followed by the path is misparsed.
pub fn output_arg(out: &Path) -> String {
    format!("/Fe{}", out.display())
}

/// Default library search path argument (harmless placeholder on Windows).
pub fn default_lib_path_arg() -> String {
    "/LIBPATH:/usr/local/lib".to_string()
}

/// Map a single GCC/Clang-style flag to its MSVC equivalent.
///
/// The input must already have its `-std=` flag stripped by the caller.
pub fn map_gnuc_to_msvc(flag: &str) -> String {
    match flag {
        "-O0" => "/Od".to_string(),
        "-O1" => "/O1".to_string(),
        "-O2" | "-O3" | "-Os" => "/O2".to_string(),
        "-g" => "/Zi".to_string(),
        "-Wall" | "-Wextra" => "/W4".to_string(),
        _ => {
            if let Some(rest) = flag.strip_prefix("-I") {
                format!("/I{rest}")
            } else if let Some(rest) = flag.strip_prefix("-L") {
                format!("/LIBPATH:{rest}")
            } else if let Some(rest) = flag.strip_prefix("-l") {
                format!("{rest}.lib")
            } else {
                flag.to_string()
            }
        }
    }
}
