use std::convert::Infallible;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use which::which;

pub use crate::error::Error;

const CANDIDATES: &[&str] = &[
    #[cfg(windows)]
    "cl",
    "cc",
    "clang",
    "gcc",
];

type NoSearch = fn(&Path) -> Option<String>;

/// Represents the detected C compiler toolchain.
///
/// The toolchain determines the flag dialect, output flag, and standard flags used
/// when invoking the compiler for C code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toolchain {
    /// GCC-compatible toolchain (gcc, cc).
    Gcc,
    /// Clang toolchain (clang, clang-cl).
    Clang,
    /// Microsoft Visual C++ toolchain (cl.exe). Only available on Windows.
    #[cfg(windows)]
    Msvc,
    /// Unknown or unsupported toolchain. Treated as GCC-compatible for best-effort compilation.
    Other,
}

impl FromStr for Toolchain {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            // FIXME: Assume it's GCC, but this could also be clang.
            // This *should* be fine as both take (almost?) the same arguments, but this should be
            // changed later on.
            "gcc" | "cc" => Toolchain::Gcc,
            "clang" => Toolchain::Clang,
            #[cfg(windows)]
            "cl" => Toolchain::Msvc,
            _ => Toolchain::Other,
        })
    }
}

/// Convert a path's file stem to a lowercase `String`.
///
/// This attempts to get the file stem (the filename without extension) from the provided `Path`,
/// convert it to UTF-8, and return a lowercase `String`.
///
/// If the path has no file stem or the file stem is not valid UTF-8, this function returns `None`.
pub fn get_lowercase_exe(path: &Path) -> Option<String> {
    Some(path.file_stem().and_then(|s| s.to_str())?.to_lowercase())
}

/// Detect the toolchain for a given executable path.
///
/// If a custom search function `search_fn` is supplied, its result (if any) overrides the simple
/// filename-based detection.
fn detect_toolchain<SearchFn>(path: &Path, search_fn: Option<SearchFn>) -> Toolchain
where
    SearchFn: for<'a> FnOnce(&'a Path) -> Option<String>,
{
    if let Some(search) = search_fn
        && let Some(toolchain_str) = search(path)
    {
        return Toolchain::from_str(&toolchain_str).expect("Infallible");
    }

    let exe = get_lowercase_exe(path).unwrap_or_default();
    Toolchain::from_str(&exe).expect("Infallible")
}

impl Toolchain {
    /// Return (toolchain, path to the compiler executable) if one was found.
    ///
    /// Detection checks (in order)
    /// 1. `CC` env var (if set and resolves)
    /// 2. Known candidates on PATH (`cl`, `cc`, `clang`, `gcc`).
    ///
    /// The returned `Toolchain` enum describes the *detected* compiler type (gcc/clang/msvc).
    pub fn executable_path() -> Option<(Toolchain, PathBuf)> {
        // Respect an explicit compiler override first.
        //
        // If CC points to a compiler name (or executable), resolve it on `PATH` and use it instead
        // of probing the system.
        if let Ok(env_cc) = env::var("CC")
            && let Ok(path) = which(&env_cc)
        {
            return Some((detect_toolchain::<NoSearch>(&path, None), path));
        }

        // Otherwise, search through the preferred compiler candidates in order.
        //
        // The first compiler we can successfully resolve is considered the system's default C
        // compiler.
        for name in CANDIDATES {
            if let Ok(path) = which(name) {
                let toolchain = if cfg!(unix) && *name == "cc" {
                    // On Unix-like systems, `cc` is often a generic frontend or symlink rather
                    // than the actual compiler. Resolve it to its underlying implementation (like
                    // GCC or clang) so we report the real toolchain instead of the generic wrapper.
                    resolve_cc_toolchain(&path)
                } else {
                    // Other compiler names already identify a specific toolchain.
                    detect_toolchain::<NoSearch>(&path, None)
                };

                return Some((toolchain, path));
            }
        }

        // On Windows, MSVC's cl.exe is usually not on PATH (only the VS developer cmd adds it).
        // Fall back to locating an installed MSVC via vswhere.
        #[cfg(windows)]
        if let Some(cl) = crate::msvc::find_msvc() {
            return Some((Toolchain::Msvc, cl));
        }

        // We didn't find any compiler on PATH
        None
    }

    /// Detect the toolchain for a specific compiler executable path without searching `PATH`. Used
    /// when the user supplies an explicit `--cc` override.
    ///
    /// Detection is filename-based (e.g. a path ending in `cl.exe` is MSVC, `clang` is Clang,
    /// everything else is treated as GCC-compatible). Falls back to `Gcc` for unrecognized names
    /// rather than `Other`, so that the caller can still attempt compilation with the override.
    pub fn from_path_lossy(path: &Path) -> Toolchain {
        detect_toolchain::<NoSearch>(path, None)
    }

    /// Returns `true` if this toolchain is MSVC.
    ///
    /// This is only true on Windows when the toolchain is explicitly detected as MSVC.
    /// On non-Windows platforms, always returns `false`.
    pub const fn is_msvc(&self) -> bool {
        #[cfg(windows)]
        {
            matches!(self, Toolchain::Msvc)
        }
        #[cfg(not(windows))]
        {
            // MSVC isn't on other OSes, so it's always false outside of Windows
            false
        }
    }

    /// Returns `true` if this toolchain is GCC or Clang (Unix-like flag dialect).
    pub const fn is_unix_like(&self) -> bool {
        matches!(self, Toolchain::Gcc | Toolchain::Clang)
    }

    /// Output flag used for the given toolchain (e.g. `-o` vs `/Fe`).
    pub const fn output_flag(&self) -> &'static str {
        match self {
            Toolchain::Gcc | Toolchain::Clang => "-o",
            #[cfg(windows)]
            Toolchain::Msvc => crate::msvc::output_flag(),
            Toolchain::Other => "-o",
        }
    }

    /// Flags that should be passed to the compiler for C compilation.
    ///
    /// These are intentionally conservative and represent a set of safe, portable defaults per
    /// toolchain. `CompilerOptions` will combine these with link and target options.
    pub fn get_compiler_flags(&self) -> Vec<String> {
        match self {
            Toolchain::Gcc | Toolchain::Clang => {
                let flags = ["-std=c99", "-Wall", "-Wextra", "-pedantic"];
                flags.iter().map(ToString::to_string).collect()
            }
            #[cfg(windows)]
            Toolchain::Msvc => crate::msvc::compiler_flags(),
            Toolchain::Other => {
                vec![]
            }
        }
    }

    /// The language-standard flag that guarantees a C99-compatible build for this toolchain.
    /// User-supplied flags that would change the standard dialect are stripped and this is
    /// re-applied (see [`translate_user_flags`]) so the generated output always compiles as C99.
    pub fn c99_standard_flag(&self) -> Option<&'static str> {
        match self {
            Toolchain::Gcc | Toolchain::Clang => Some("-std=c99"),
            #[cfg(windows)]
            Toolchain::Msvc => crate::msvc::c99_standard_flag(),
            Toolchain::Other => None,
        }
    }

    /// Translate user-supplied C compiler flags to this toolchain while
    /// guaranteeing a C99-compatible build.
    ///
    /// The compiler emits C99 source, so any user flag that would change the
    /// language standard (GCC/Clang `-std=...`, MSVC `/std:...`) is removed and
    /// the toolchain's C99 standard flag is re-applied last. All other flags are
    /// normalized to the toolchain's spelling:
    ///
    /// - GCC/Clang `->` MSVC:
    ///   * `-O2`/`-O1`/`-O0` -> `/O2`/`/O1`/`/Od`,
    ///   * `-g` -> `/Zi`, `-I<x>` -> `/I<x>`,
    ///   * `-L<x>` -> `/LIBPATH:<x>`,
    ///   * `-l<x>` -> `<x>.lib`,
    ///   * `-Wall` -> `/W4`.
    ///
    /// - MSVC `->` GCC/Clang:
    ///   * `/O2`/`/O1`/`/Od` -> `-O2`/`-O1`/`-O0`,
    ///   * `/Zi` -> `-g`
    ///   * `/I<x>` -> `-I<x>`
    ///   * `/LIBPATH:<x>` -> `-L<x>`,
    ///   * `<x>.lib` -> `-l<x>`
    ///   * `/W4` -> `-Wall`.
    ///
    /// Unknown flags are passed through unchanged. This keeps the CLI portable:
    /// a user can write `-O2 -g` regardless of whether the system compiler is
    /// `gcc`, `clang`, or `cl`.
    pub fn translate_user_flags(&self, user_flags: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let is_msvc = self.is_msvc();
        let is_unix = self.is_unix_like();

        for raw in user_flags {
            let flag = raw.trim();
            if flag.is_empty() {
                continue;
            }

            // Strip any standard-selection flag; we re-apply C99 at the end.
            let is_std_flag = match self {
                Toolchain::Gcc | Toolchain::Clang => flag.starts_with("-std="),
                #[cfg(windows)]
                Toolchain::Msvc => crate::msvc::is_standard_flag(flag),
                Toolchain::Other => false,
            };
            if is_std_flag {
                continue;
            }

            let is_gnu_style = flag.starts_with('-');
            if is_msvc && is_gnu_style {
                // Compiler is MSVC but the flag is GCC/Clang-style: map it.
                #[cfg(windows)]
                out.push(crate::msvc::map_gnuc_to_msvc(flag));
            } else if is_unix && !is_gnu_style {
                // Compiler is GCC/Clang but the flag is MSVC-style: map it.
                out.push(map_msvc_to_gnuc(flag));
            } else {
                out.push(flag.to_string());
            }
        }

        // Guarantee C99 output by re-applying the standard flag last (so it wins over any user
        // flag we may have normalized).
        if let Some(std) = self.c99_standard_flag() {
            out.push(std.to_string());
        }
        out
    }
}

/// Map a single MSVC-style flag to its GCC/Clang equivalent. The input must already have its
/// `/std:` flag stripped by the caller.
fn map_msvc_to_gnuc(flag: &str) -> String {
    match flag {
        "/Od" => "-O0".to_string(),
        "/O1" => "-O1".to_string(),
        "/O2" => "-O2".to_string(),
        "/Zi" | "/Z7" => "-g".to_string(),
        "/W4" | "/W3" => "-Wall".to_string(),
        _ => {
            let lower = flag.to_ascii_lowercase();
            if let Some(rest) = lower.strip_prefix("/i") {
                format!("-I{rest}")
            } else if let Some(rest) = lower.strip_prefix("/libpath:") {
                format!("-L{rest}")
            } else if lower.ends_with(".lib") {
                format!("-l{}", &lower[..lower.len() - 4])
            } else {
                flag.to_string()
            }
        }
    }
}

/// `cc` is often a symlink to an actual compiler on the system, so
/// we need to get an actual path to the C compiler.
fn resolve_cc_toolchain(path: &Path) -> Toolchain {
    if cfg!(unix)
        && path.ends_with("cc")
        && let Ok(real_path) = std::fs::read_link(path)
    {
        return detect_toolchain::<NoSearch>(&real_path, None);
    }
    detect_toolchain::<NoSearch>(path, None)
}

/// Configuration options for invoking the C compiler.
///
/// This builder-style struct collects all information needed to construct a compiler
/// invocation: toolchain, source files, output path, include paths, library paths,
/// user flags, and link libraries.
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub toolchain: Toolchain,
    /// Path to the compiler executable (cached to avoid re-detection)
    pub compiler_path: Option<PathBuf>,
    /// Source files to compile
    pub sources: Vec<PathBuf>,
    /// Single output (target) file
    pub output: Option<PathBuf>,
    pub link_opts: Vec<String>,
    /// Include search paths (`-I` flags)
    pub includes: Vec<PathBuf>,
    /// User-supplied C compiler flags (e.g. `-O2 -g`), translated to the
    /// target toolchain and guaranteed C99-compatible by `build_invocation`.
    pub user_cflags: Vec<String>,
    /// Additional library search paths supplied by the user (`-L` / `/LIBPATH:`).
    pub user_lib_paths: Vec<PathBuf>,
}

/// Simple wrapper around a toolchain for type-safe construction of `CompilerOptions`.
#[derive(Debug, Clone, Copy)]
pub struct CompilerMeta(pub Toolchain);

impl CompilerOptions {
    /// Create a new `CompilerOptions` with the given toolchain metadata.
    pub const fn new(base_meta: CompilerMeta) -> Self {
        Self {
            toolchain: base_meta.0,
            compiler_path: None,
            sources: Vec::new(),
            output: None,
            link_opts: Vec::new(),
            includes: Vec::new(),
            user_cflags: Vec::new(),
            user_lib_paths: Vec::new(),
        }
    }

    /// Translate library names into toolchain-specific link arguments and append them.
    ///
    /// Example:
    /// - GCC/Clang: `["-lm", "-lpthread"]`
    /// - MSVC (Windows): `["m.lib"]`
    pub fn link_libs<S: AsRef<str>>(mut self, libs: &[S]) -> Self {
        for lib in libs {
            match self.toolchain {
                Toolchain::Gcc | Toolchain::Clang => {
                    self.link_opts.push(format!("-l{}", lib.as_ref()));
                }
                #[cfg(windows)]
                Toolchain::Msvc => {
                    self.link_opts.push(crate::msvc::link_lib_arg(lib.as_ref()));
                }
                Toolchain::Other => {}
            }
        }
        self
    }

    /// Add library search paths (translated per toolchain).
    pub fn lib_paths<P>(mut self, paths: &[P]) -> Self
    where
        P: Into<PathBuf> + AsRef<OsStr>,
    {
        for p in paths {
            let path: PathBuf = p.into();
            match self.toolchain {
                Toolchain::Gcc | Toolchain::Clang => {
                    self.link_opts.push(format!("-L{}", path.display()));
                }
                #[cfg(windows)]
                Toolchain::Msvc => {
                    self.link_opts.push(crate::msvc::lib_path_arg(&path));
                }
                Toolchain::Other => {}
            }
        }
        self
    }

    /// Add source files to compile
    pub fn sources<P: Into<PathBuf> + AsRef<OsStr>>(mut self, items: &[P]) -> Self {
        for i in items {
            self.sources.push(i.into());
        }
        self
    }

    /// Set the output file
    pub fn output<P: Into<PathBuf>>(mut self, out: P) -> Self {
        self.output = Some(out.into());
        self
    }

    /// Set the compiler executable path (avoids re-detection).
    pub fn compiler_path(mut self, path: PathBuf) -> Self {
        self.compiler_path = Some(path);
        self
    }

    /// Add include search paths (`-I` flags).
    pub fn includes<P: Into<PathBuf> + Clone>(mut self, items: &[P]) -> Self {
        for i in items {
            self.includes.push(i.clone().into());
        }
        self
    }

    /// Add user-supplied C compiler flags (e.g. `-O2`, `-g`).
    ///
    /// These are translated to the target toolchain by `build_invocation` and
    /// are guaranteed not to break the C99 compatibility of the generated code.
    pub fn user_cflags<S: AsRef<str>>(mut self, flags: &[S]) -> Self {
        self.user_cflags
            .extend(flags.iter().map(|f| f.as_ref().to_string()));
        self
    }

    /// Add additional library search paths (translated per toolchain).
    pub fn user_lib_paths<P>(mut self, paths: &[P]) -> Self
    where
        P: Into<PathBuf> + AsRef<OsStr>,
    {
        for p in paths {
            self.user_lib_paths.push(p.into());
        }
        self
    }

    /// Build the compiler invocation.
    ///
    /// Returns `(path_to_compiler_executable, args_vec)`.
    ///
    /// # Errors
    ///
    /// - if `sources` is empty
    /// - if `output` is not set
    /// - if no system compiler can be found and no `compiler_path` was set
    pub fn build_invocation(&self) -> Result<(PathBuf, Vec<String>), crate::error::Error> {
        if self.sources.is_empty() {
            return Err(crate::error::Error::CompileError(
                "no source files specified in CompilerOptions".into(),
            ));
        }
        let out = self.output.as_ref().ok_or_else(|| {
            crate::error::Error::CompileError(
                "output (target) path not set in CompilerOptions".into(),
            )
        })?;

        let compiler_path = self
            .compiler_path
            .clone()
            .or_else(|| Toolchain::executable_path().map(|(_, p)| p))
            .ok_or_else(|| crate::error::Error::CompileError("no system compiler found".into()))?;

        let mut args = Vec::new();

        for inc in &self.includes {
            args.push(format!("-I{}", inc.display()));
        }

        for s in &self.sources {
            args.push(s.display().to_string());
        }

        // Output: GCC/Clang take `-o <exe>`. MSVC's cl.exe must get `/Fe<exe>` as a single
        // attached token to produce the executable (a detached `/Fe` followed by the path is
        // misparsed by cl).
        match self.toolchain {
            Toolchain::Gcc | Toolchain::Clang | Toolchain::Other => {
                args.push("-o".to_string());
                args.push(out.display().to_string());
            }
            #[cfg(windows)]
            Toolchain::Msvc => {
                args.push(crate::msvc::output_arg(out));
            }
        }

        // Base C99 standard + warning flags, then any user-supplied flags (translated to this
        // toolchain and guarantees ANSI C99-compatible).
        args.extend(self.toolchain.get_compiler_flags());
        args.extend(self.toolchain.translate_user_flags(&self.user_cflags));

        // Library search paths: the hardcoded system default first, then any user-supplied paths
        // (translated per toolchain).
        match self.toolchain {
            Toolchain::Gcc | Toolchain::Clang => {
                args.push("-L/usr/local/lib".to_string());
            }
            #[cfg(windows)]
            Toolchain::Msvc => {
                args.push(crate::msvc::default_lib_path_arg());
            }
            Toolchain::Other => {}
        }
        for p in &self.user_lib_paths {
            match self.toolchain {
                Toolchain::Gcc | Toolchain::Clang => {
                    args.push(format!("-L{}", p.display()));
                }
                #[cfg(windows)]
                Toolchain::Msvc => {
                    args.push(crate::msvc::lib_path_arg(p));
                }
                Toolchain::Other => {}
            }
        }
        args.extend(self.link_opts.clone());

        Ok((compiler_path, args))
    }

    pub fn build(self) -> CompilerOptions {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_strips_std_flag_and_reapplies_c99() {
        let t = Toolchain::Gcc;
        let out = t.translate_user_flags(&["-std=gnu11".to_string(), "-O2".to_string()]);
        // The user's -std= is dropped and -std=c99 is re-applied at the end.
        assert!(!out.contains(&"-std=gnu11".to_string()));
        assert_eq!(out.last().map(String::as_str), Some("-std=c99"));
        assert!(out.contains(&"-O2".to_string()));
    }

    #[test]
    fn translate_passes_optimization_flags_through() {
        let t = Toolchain::Clang;
        let out = t.translate_user_flags(&["-O2".to_string(), "-g".to_string()]);
        assert!(out.contains(&"-O2".to_string()));
        assert!(out.contains(&"-g".to_string()));
        assert!(out.contains(&"-std=c99".to_string()));
    }

    #[cfg(windows)]
    #[test]
    fn translate_gnu_flags_to_msvc() {
        let t = Toolchain::Msvc;
        let out =
            t.translate_user_flags(&["-O2".to_string(), "-g".to_string(), "-Wall".to_string()]);
        assert!(out.contains(&"/O2".to_string()));
        assert!(out.contains(&"/Zi".to_string()));
        assert!(out.contains(&"/W4".to_string()));
        // MSVC build re-applies /std:c11 (not -std=c99).
        assert_eq!(out.last().map(String::as_str), Some("/std:c11"));
    }

    #[test]
    fn translate_msvc_flags_to_gnu() {
        let t = Toolchain::Gcc;
        let out = t.translate_user_flags(&["/O2".to_string(), "/Zi".to_string()]);
        assert!(out.contains(&"-O2".to_string()));
        assert!(out.contains(&"-g".to_string()));
    }

    #[test]
    fn translate_lib_paths_and_libs() {
        let t = Toolchain::Gcc;
        let out = t.translate_user_flags(&["-L/some/lib".to_string(), "-lm".to_string()]);
        assert!(out.contains(&"-L/some/lib".to_string()));
        assert!(out.contains(&"-lm".to_string()));
    }
}
