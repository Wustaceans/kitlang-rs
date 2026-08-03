use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use crate::compiler::Toolchain;

/// Information about the detected C compiler, including its system include paths.
///
/// This struct is populated at startup by [`init_compiler_info`] and cached globally.
/// It contains the detected toolchain type, compiler executable path, system include
/// directories, target triple (if cross-compiling), and a flag indicating whether
/// cross-compilation is in effect.
#[derive(Debug, Clone)]
pub struct CompilerInfo {
    /// The detected compiler toolchain (GCC, Clang, MSVC, etc.).
    pub toolchain: Toolchain,
    /// Absolute path to the compiler executable.
    pub compiler_path: PathBuf,
    /// System include directories discovered by querying the compiler.
    pub system_include_dirs: Vec<PathBuf>,
    /// Target triple if cross-compiling (e.g., "aarch64-linux-gnu").
    pub target_triple: Option<String>,
    /// True if the compiler targets a different architecture/OS than the host.
    pub is_cross_compiling: bool,
}

static COMPILER_INFO: OnceLock<Option<CompilerInfo>> = OnceLock::new();

/// Initialize compiler detection at startup.
///
/// Detects the system C compiler (respecting `KITC_CC`, `CC` env vars and PATH),
/// queries it for built-in system include paths, and caches the result globally.
///
/// Returns the detected [`CompilerInfo`], or `None` if no compiler was found
/// (in which case an error message is printed to stderr).
///
/// This should be called once at program startup to fail fast if no C compiler is available.
pub fn init_compiler_info() -> Option<CompilerInfo> {
    let info = detect_compiler();
    COMPILER_INFO.set(info.clone()).ok();
    info
}

/// Get the previously initialized compiler info.
///
/// Returns `None` if [`init_compiler_info`] has not been called yet, or if it returned `None`.
pub fn get_compiler_info() -> Option<&'static CompilerInfo> {
    COMPILER_INFO.get().and_then(|opt| opt.as_ref())
}

/// Get system include directories from the detected compiler.
///
/// Returns an empty vector if the compiler has not been initialized yet.
pub fn get_system_include_dirs() -> Vec<PathBuf> {
    get_compiler_info()
        .map(|ci| ci.system_include_dirs.clone())
        .unwrap_or_default()
}

/// Detect the system compiler and query its include paths.
fn detect_compiler() -> Option<CompilerInfo> {
    // 1. Check KITC_CC env var for explicit override
    if let Ok(cc) = env::var("KITC_CC") {
        let path = PathBuf::from(&cc);
        if let Some(info) = try_compiler(&path) {
            return Some(info);
        }
    }

    // 2. Check CC env var
    if let Ok(cc) = env::var("CC")
        && let Ok(path) = which::which(&cc)
        && let Some(info) = try_compiler(&path)
    {
        return Some(info);
    }

    // 3. Search for candidates on PATH
    let candidates = get_candidates();
    for name in candidates {
        if let Ok(path) = which::which(name)
            && let Some(info) = try_compiler(&path)
        {
            return Some(info);
        }
    }

    // No compiler found
    eprintln!(
        "Error: No C compiler found. Please install gcc, clang, or MSVC and ensure it's in PATH."
    );
    eprintln!("       You can also set KITC_CC to specify a compiler explicitly.");
    None
}

fn get_candidates() -> Vec<&'static str> {
    #[cfg(windows)]
    {
        vec!["cl.exe", "clang-cl.exe", "cc", "clang", "gcc"]
    }
    #[cfg(not(windows))]
    {
        vec!["cc", "clang", "gcc"]
    }
}

fn try_compiler(path: &Path) -> Option<CompilerInfo> {
    let toolchain = detect_toolchain(path);
    let system_include_dirs = match toolchain {
        Toolchain::Gcc | Toolchain::Clang => query_gcc_like_includes(path),
        #[cfg(windows)]
        Toolchain::Msvc => get_msvc_includes(),
        Toolchain::Other => query_gcc_like_includes(path),
    };

    let target_triple = detect_target_triple(&toolchain, path);
    let is_cross_compiling = target_triple.as_ref().is_some_and(|t| {
        !t.contains(std::env::consts::ARCH)
            || (cfg!(windows) && !t.contains("windows"))
            || (!cfg!(windows) && t.contains("windows"))
    });

    Some(CompilerInfo {
        toolchain,
        compiler_path: path.to_path_buf(),
        system_include_dirs,
        target_triple,
        is_cross_compiling,
    })
}

fn detect_toolchain(path: &Path) -> Toolchain {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match name.as_str() {
        "gcc" | "cc" => Toolchain::Gcc,
        "clang" => Toolchain::Clang,
        #[cfg(windows)]
        "cl" => Toolchain::Msvc,
        #[cfg(windows)]
        "clang-cl" => Toolchain::Clang, // Treat clang-cl as Clang for include purposes
        _ => Toolchain::Other,
    }
}

/// Queries a GCC/Clang-compatible compiler for its built-in system include paths.
///
/// Runs `compiler -E -Wp,-v -xc /dev/null` and parses the stderr output to extract
/// the system include search paths. This is used for GCC/Clang toolchains and as a
/// best-effort fallback for unknown toolchains (Toolchain::Other).
fn query_gcc_like_includes(compiler: &Path) -> Vec<PathBuf> {
    let output = Command::new(compiler)
        .args(["-E", "-Wp,-v", "-xc", "/dev/null"])
        .stderr(std::process::Stdio::piped())
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    parse_include_paths(&stderr)
}

fn parse_include_paths(stderr: &str) -> Vec<PathBuf> {
    let mut in_section = false;
    let mut paths = Vec::new();

    for line in stderr.lines() {
        let trimmed = line.trim();

        if trimmed.contains("#include") && trimmed.contains("search starts here") {
            in_section = true;
            continue;
        }

        if trimmed == "End of search list." {
            in_section = false;
            continue;
        }

        if in_section && !trimmed.is_empty() {
            // Skip framework directories on macOS for now
            if trimmed.contains("(framework directory)") {
                continue;
            }
            paths.push(PathBuf::from(trimmed));
        }
    }

    paths
}

#[cfg(windows)]
fn get_msvc_includes() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. Check INCLUDE environment variable (set by vcvarsall.bat)
    if let Ok(include) = env::var("INCLUDE") {
        for dir in include.split(';') {
            let dir = dir.trim();
            if !dir.is_empty() {
                paths.push(PathBuf::from(dir));
            }
        }
    }

    // 2. Try vswhere to find Visual Studio installation
    if let Some(vs_paths) = find_vs_includes() {
        paths.extend(vs_paths);
    }

    // 3. Check for Windows SDK paths
    if let Some(sdk_paths) = find_windows_sdk_includes() {
        paths.extend(sdk_paths);
    }

    paths
}

#[cfg(windows)]
fn find_vs_includes() -> Option<Vec<PathBuf>> {
    // Try vswhere.exe
    let vswhere_path = find_vswhere()?;
    let output = Command::new(vswhere_path)
        .args([
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
        ])
        .output()
        .ok()?;

    let install_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if install_path.is_empty() {
        return None;
    }

    let mut paths = Vec::new();

    // VC++ tools include directory
    let vc_tools = PathBuf::from(&install_path).join("VC/Tools/MSVC");
    if let Ok(entries) = std::fs::read_dir(&vc_tools) {
        let mut versions: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        versions.sort_by(|a, b| b.file_name().cmp(&a.file_name())); // Latest first
        if let Some(latest) = versions.first() {
            paths.push(latest.path().join("include"));
        }
    }

    Some(paths)
}

#[cfg(windows)]
fn find_vswhere() -> Option<PathBuf> {
    // Standard location
    let standard =
        PathBuf::from(r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe");
    if standard.exists() {
        return Some(standard);
    }

    // Check ProgramFiles(x86) env var
    if let Ok(pf) = env::var("ProgramFiles(x86)") {
        let path = PathBuf::from(pf).join("Microsoft Visual Studio/Installer/vswhere.exe");
        if path.exists() {
            return Some(path);
        }
    }

    // Check ProgramFiles env var
    if let Ok(pf) = env::var("ProgramFiles") {
        let path = PathBuf::from(pf).join("Microsoft Visual Studio/Installer/vswhere.exe");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

#[cfg(windows)]
fn find_windows_sdk_includes() -> Option<Vec<PathBuf>> {
    let mut paths = Vec::new();

    // Check Windows SDK environment variables
    if let Ok(sdk_dir) = env::var("WindowsSdkDir") {
        let sdk = PathBuf::from(sdk_dir);
        paths.push(sdk.join("Include").join("ucrt"));
        paths.push(sdk.join("Include").join("um"));
        paths.push(sdk.join("Include").join("shared"));
        paths.push(sdk.join("Include").join("winrt"));
        paths.push(sdk.join("Include").join("cppwinrt"));
    }

    // Check for 10.0.* versioned SDK
    if let Ok(program_files) = env::var("ProgramFiles(x86)") {
        let sdk_root = PathBuf::from(program_files).join("Windows Kits/10/Include");
        if sdk_root.exists() {
            if let Ok(entries) = std::fs::read_dir(&sdk_root) {
                let mut versions: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
                if let Some(latest) = versions.first() {
                    paths.push(latest.path().join("ucrt"));
                    paths.push(latest.path().join("um"));
                    paths.push(latest.path().join("shared"));
                    paths.push(latest.path().join("winrt"));
                    paths.push(latest.path().join("cppwinrt"));
                }
            }
        }
    }

    if paths.is_empty() { None } else { Some(paths) }
}

fn detect_target_triple(toolchain: &Toolchain, compiler: &Path) -> Option<String> {
    // Check if compiler name has target prefix (e.g., aarch64-linux-gnu-gcc)
    let name = compiler.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    // Common cross-compiler prefixes
    let prefixes = [
        "aarch64-linux-gnu-",
        "arm-linux-gnueabihf-",
        "x86_64-w64-mingw32-",
        "i686-w64-mingw32-",
        "riscv64-linux-gnu-",
        "powerpc64le-linux-gnu-",
        "s390x-linux-gnu-",
    ];

    for prefix in &prefixes {
        if name.starts_with(prefix) {
            return Some(prefix.trim_end_matches('-').to_string());
        }
    }

    // For clang, we could try -print-target-triple
    if matches!(toolchain, Toolchain::Clang) {
        let output = Command::new(compiler)
            .args(["-print-target-triple"])
            .output()
            .ok()?;
        let triple = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !triple.is_empty() && triple != "unknown-unknown-unknown" {
            return Some(triple);
        }
    }

    None
}

/// Returns the minimal set of builtin headers that Kit bundles.
///
/// These headers (stddef.h, stdarg.h, stdint.h, stdbool.h, limits.h, float.h,
/// inttypes.h) match the minimal set that Clang provides in its resource directory.
/// They are embedded at compile time via `include_str!` and used by the includium
/// preprocessor when resolving system includes like `<stddef.h>`.
///
/// The headers are loaded from `kitc-common/resources/include/` at compile time.
pub fn get_builtin_headers() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("stddef.h", include_str!("../resources/include/stddef.h"));
    map.insert("stdarg.h", include_str!("../resources/include/stdarg.h"));
    map.insert("stdint.h", include_str!("../resources/include/stdint.h"));
    map.insert("stdbool.h", include_str!("../resources/include/stdbool.h"));
    map.insert("limits.h", include_str!("../resources/include/limits.h"));
    map.insert("float.h", include_str!("../resources/include/float.h"));
    map.insert(
        "inttypes.h",
        include_str!("../resources/include/inttypes.h"),
    );
    map
}
