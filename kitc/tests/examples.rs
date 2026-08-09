//! End-to-end compiler tests driven by the Kit fixture corpus in `examples/`.
//!
//! The individual `#[test]` functions are **generated at build time** by `kitc/build.rs` from the
//! fixtures on disk (see `{OUT_DIR}/example_tests.rs`). Each fixture becomes its own test, so
//! `cargo test` can filter per category:
//!
//! ```text
//! cargo test -p kitc --test examples -- struct
//! cargo test -p kitc --test examples -- range
//! ```
//!
//! Fixture kinds (see `build.rs` for the discovery rules):
//!
//! * `RunCompare`: `*.kit` with a sibling `*.kit.expected`; compiled, run, stdout compared. Optional
//!   stdin is read from a `<name>.kit.stdin` sidecar file.
//! * `CompileFail`: `*.kit` under `examples/negative/`; must *fail* to compile.

use std::{
    error::Error,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

/// The kinds of fixtures the generated tests exercise.
#[derive(Clone, Copy)]
enum FixtureKind {
    RunCompare,
    CompileFail,
}

fn exe_ext() -> &'static str {
    if cfg!(windows) { "exe" } else { "" }
}

/// Normalize CRLF line endings so expected files are platform-independent.
fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("kitc crate should be a member of the workspace")
        .to_path_buf()
}

/// Entry point invoked by every generated `#[test]` in `{OUT_DIR}/example_tests.rs`.
fn run_fixture(
    rel_path: &str,
    stdin: Option<&str>,
    kind: FixtureKind,
    source_path: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let root = workspace_root();
    let source_path_full = root.join("examples").join(rel_path);

    assert!(
        source_path_full.exists(),
        "fixture {} does not exist",
        source_path_full.display()
    );

    let kitc = env!("CARGO_BIN_EXE_kitc");

    match kind {
        FixtureKind::CompileFail => {
            assert_compile_fails(kitc, &source_path_full, source_path, &root)
        }
        FixtureKind::RunCompare => {
            let expected_path = source_path_full.with_extension("kit.expected");
            run_and_compare(
                kitc,
                &source_path_full,
                &expected_path,
                stdin,
                source_path,
                &root,
            )
        }
    }
}

fn compile(
    kitc: &str,
    source: &Path,
    source_path: Option<&str>,
    cwd: &Path,
) -> Result<Output, Box<dyn Error>> {
    let mut cmd = Command::new(kitc);
    cmd.current_dir(cwd);
    cmd.arg("compile");
    if let Some(sp) = source_path {
        cmd.arg("--source-path").arg(sp);
    }
    cmd.arg(source);
    cmd.output().map_err(Into::into)
}

fn run_and_compare(
    kitc: &str,
    source: &Path,
    expected_path: &Path,
    stdin: Option<&str>,
    source_path: Option<&str>,
    cwd: &Path,
) -> Result<(), Box<dyn Error>> {
    let output = compile(kitc, source, source_path, cwd)?;
    assert!(
        output.status.success(),
        "kitc failed to compile {}\n{}",
        source.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    let exe_path = source.with_extension(exe_ext());
    let run_output = run_executable(&exe_path, stdin)?;
    // Remove the executable eagerly so a failing assertion below cannot leave it behind.
    let _ = std::fs::remove_file(&exe_path);

    let expected = normalize_line_endings(&std::fs::read_to_string(expected_path)?);
    let actual = normalize_line_endings(&String::from_utf8_lossy(&run_output.stdout));
    assert_eq!(
        actual,
        expected,
        "stdout mismatch for {}. Expected:\n{}\n---\nActual:\n{}",
        source.display(),
        expected,
        actual
    );

    Ok(())
}

fn assert_compile_fails(
    kitc: &str,
    source: &Path,
    source_path: Option<&str>,
    cwd: &Path,
) -> Result<(), Box<dyn Error>> {
    let output = compile(kitc, source, source_path, cwd)?;
    assert!(
        !output.status.success(),
        "expected {} to fail compilation but it succeeded:\n{}",
        source.display(),
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(())
}

fn run_executable(exe: &Path, stdin: Option<&str>) -> Result<Output, Box<dyn Error>> {
    let mut cmd = Command::new(exe);
    cmd.stdin(if stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    cmd.stdout(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to launch executable {}: {e}", exe.display()))?;

    if let Some(data) = stdin {
        use std::io::Write;
        child
            .stdin
            .take()
            .expect("stdin pipe requested")
            .write_all(data.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{} exited with {}",
        exe.display(),
        output.status
    );
    Ok(output)
}

// The per-fixture tests are generated at build time from the `examples/` corpus.
include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
