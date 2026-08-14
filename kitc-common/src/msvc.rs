//! MSVC (Microsoft Visual C++) toolchain support.
//!
//! This module centralises all Windows/MSVC-specific logic:
//!
//! - compiler discovery;
//! - build-environment capture;
//! - include-path discovery;
//! - flag translation for `cl.exe`.

use find_msvc_tools::Tool as MsvcTool;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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

/// System include directories for MSVC.
///
/// The toolchain's `INCLUDE` environment variable is preferred because it
/// matches exactly what `cl.exe` would search. Runners that are not launched
/// from a Visual Studio developer prompt often have an empty `INCLUDE`, so when
/// the tool path yields nothing we fall back to scanning the filesystem for the
/// standard MSVC and Windows SDK header roots ([`manual_include_dirs`]).
pub fn get_includes() -> Vec<PathBuf> {
    let tool = get_tool().and_then(|tool| {
        tool.env()
            .into_iter()
            .find(|(k, _)| k.to_string_lossy().as_ref() == "INCLUDE")
            .map(|(_, v)| env::split_paths(v).collect::<Vec<_>>())
    });
    match tool {
        Some(dirs) if !dirs.is_empty() => dirs,
        _ => manual_include_dirs(),
    }
}

/// Discover include directories straight from the filesystem.
///
/// This is independent of the developer-shell environment. It locates the two
/// standard header roots that make up an MSVC `INCLUDE` value: the toolchain
/// `include` directory (vcruntime.h, sal.h, ...) and the Windows SDK `ucrt`,
/// `um`, and `shared` directories (stdio.h, stdlib.h, sal.h, ...). Each root is
/// enumerated and the highest installed version is chosen.
pub(crate) fn manual_include_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(inc) = latest_msvc_include_dir() {
        dirs.push(inc);
    }
    dirs.extend(latest_windows_sdk_include_dirs());
    dirs
}

/// Locate the Visual Studio installation root.
///
/// Uses `vswhere` when present, then falls back to the conventional install
/// locations on disk.
fn visual_studio_root() -> Option<PathBuf> {
    let vswhere =
        Path::new("C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe");
    if vswhere.exists() {
        let out = Command::new(vswhere)
            .args([
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-property",
                "installationPath",
            ])
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let line = stdout.lines().next().unwrap_or("").trim().to_string();
        if !line.is_empty() {
            return Some(PathBuf::from(line));
        }
    }

    for root in [
        "C:\\Program Files\\Microsoft Visual Studio",
        "C:\\Program Files (x86)\\Microsoft Visual Studio",
    ] {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return Some(entry.path());
                }
            }
        }
    }
    None
}

/// The `include` directory of the highest installed MSVC toolchain version
/// (containing vcruntime.h, sal.h, and other compiler headers).
fn latest_msvc_include_dir() -> Option<PathBuf> {
    let vs = visual_studio_root()?;
    let tools = vs.join("VC").join("Tools").join("MSVC");
    let versions = fs::read_dir(&tools).ok()?;
    highest_dir(versions)
        .map(|d| d.join("include"))
        .filter(|p| p.is_dir())
}

/// The `ucrt`, `um`, and `shared` include directories of the highest installed
/// Windows SDK version (containing stdio.h, stdlib.h, sal.h, and friends).
fn latest_windows_sdk_include_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for kits in [
        "C:\\Program Files (x86)\\Windows Kits\\10\\Include",
        "C:\\Program Files\\Windows Kits\\10\\Include",
    ] {
        let Ok(versions) = fs::read_dir(kits) else {
            continue;
        };
        let Some(latest) = highest_dir(versions) else {
            continue;
        };
        for sub in ["ucrt", "um", "shared"] {
            let p = latest.join(sub);
            if p.is_dir() {
                dirs.push(p);
            }
        }
    }
    dirs
}

/// Pick the versioned subdirectory whose name sorts last (highest version).
fn highest_dir(versions: std::fs::ReadDir) -> Option<PathBuf> {
    versions
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .max_by(|a, b| a.file_name().cmp(&b.file_name()))
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

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    /// The filesystem-based discovery must locate a real MSVC toolchain include
    /// dir and a Windows SDK include dir (stdio.h / stdlib.h live in the latter).
    #[test]
    fn manual_discovery_finds_msvc_and_sdk_headers() {
        let dirs = manual_include_dirs();
        assert!(
            !dirs.is_empty(),
            "manual include discovery returned nothing"
        );

        let has_vcruntime = dirs.iter().any(|d| d.join("vcruntime.h").exists());
        let has_stdlib = dirs.iter().any(|d| d.join("stdlib.h").exists());
        let has_stdio = dirs.iter().any(|d| d.join("stdio.h").exists());

        assert!(
            has_vcruntime,
            "expected an MSVC include dir with vcruntime.h, got {dirs:?}"
        );
        assert!(
            has_stdlib && has_stdio,
            "expected a Windows SDK include dir with stdlib.h/stdio.h, got {dirs:?}"
        );
    }
}
