use std::collections::HashMap;
use std::path::Path;

use includium::{PreprocessorConfig, PreprocessorDriver};

use super::error::{FfiError, FfiResult};
use super::system_headers::FakeHeaders;

/// Configuration for the C preprocessor.
#[derive(Clone, Debug)]
pub struct PreprocessConfig {
    /// System include directories.
    pub system_include_dirs: Vec<String>,
    /// User include directories (for `#include "..."`).
    pub user_include_dirs: Vec<String>,
    /// Additional predefined macros (`-DNAME=value`).
    pub predefined_macros: HashMap<String, String>,
    /// Whether to inject fake system headers.
    pub use_fake_system_headers: bool,
    /// Target platform.
    pub target: Target,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Target {
    #[default]
    Linux,
    Windows,
    MacOS,
}

impl Default for PreprocessConfig {
    fn default() -> Self {
        Self {
            system_include_dirs: Vec::new(),
            user_include_dirs: Vec::new(),
            predefined_macros: HashMap::new(),
            use_fake_system_headers: true,
            target: Target::default(),
        }
    }
}

impl PreprocessConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system_include_dir(mut self, dir: &str) -> Self {
        self.system_include_dirs.push(dir.to_string());
        self
    }

    pub fn add_user_include_dir(mut self, dir: &str) -> Self {
        self.user_include_dirs.push(dir.to_string());
        self
    }

    pub fn define_macro(mut self, name: &str, value: &str) -> Self {
        self.predefined_macros
            .insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    pub fn with_fake_system_headers(mut self, use_fake: bool) -> Self {
        self.use_fake_system_headers = use_fake;
        self
    }
}

/// Preprocess a C header file using includium.
///
/// Returns the preprocessed source text as a string.
pub fn preprocess_header(header_path: &Path, config: &PreprocessConfig) -> FfiResult<String> {
    let source = std::fs::read_to_string(header_path).map_err(|e| FfiError::Io(e))?;

    let header_dir = header_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    preprocess_source(&source, &header_dir, config)
}

/// Preprocess a fake system header (used when real system headers are not available).
pub fn preprocess_fake_header(content: &str, config: &PreprocessConfig) -> FfiResult<String> {
    preprocess_source(content, "", config)
}

/// Preprocess a source string from memory (no file system access for the source itself).
pub fn preprocess_source_from_string(source: &str, config: &PreprocessConfig) -> FfiResult<String> {
    preprocess_source(source, "", config)
}

/// Internal: preprocess a source string with the given configuration.
fn preprocess_source(
    source: &str,
    header_dir: &str,
    config: &PreprocessConfig,
) -> FfiResult<String> {
    let mut pp = PreprocessorDriver::new();

    // Set up the includium config based on target
    let pp_config = match config.target {
        Target::Linux => PreprocessorConfig::for_linux(),
        Target::Windows => PreprocessorConfig::for_windows(),
        Target::MacOS => PreprocessorConfig::for_macos(),
    };
    pp.apply_config(&pp_config);

    // Define user-provided macros
    for (name, value) in &config.predefined_macros {
        pp.define(name, None, value, false);
    }

    // Set up include resolver
    let system_dirs = config.system_include_dirs.clone();
    let user_dirs: Vec<String> = {
        let mut dirs = vec![header_dir.to_string()];
        dirs.extend(config.user_include_dirs.clone());
        dirs
    };
    let use_fake = config.use_fake_system_headers;

    pp = pp.with_include_resolver(move |path, kind, _ctx| match kind {
        includium::IncludeKind::System => {
            if use_fake {
                let stripped = path.trim_start_matches('/');
                if let Some(content) = FakeHeaders::get(stripped) {
                    return Some(content.to_string());
                }
            }
            for dir in &system_dirs {
                let full_path = std::path::Path::new(dir).join(path);
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    return Some(content);
                }
            }
            log::warn!("System header not found: <{}>", path);
            None
        }
        includium::IncludeKind::Local => {
            for dir in &user_dirs {
                let full_path = std::path::Path::new(dir).join(path);
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    return Some(content);
                }
            }
            log::warn!("Local header not found: \"{}\"", path);
            None
        }
    });

    // Process the source
    let result = pp
        .process(source)
        .map_err(|e| FfiError::Preprocess(format!("{e}")))?;

    Ok(result)
}
