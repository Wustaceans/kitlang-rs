//! Progress reporting for compilation stages.
//!
//! This module provides a few traits to change how you show progress to the user during compilation.
//! During tests, this behavior is suppressed with [`NoOpProgress`].

use std::time::Duration;

/// Progress reporting during compilation.
///
/// Implement this trait to receive stage-start and stage-done callbacks from
/// [`frontend::Compiler::compile`].
pub trait Progress {
    /// Called when a new compilation stage starts.
    fn stage(&self, name: &str);
    /// Called when a stage finishes successfully.
    fn stage_done(&self, name: &str, elapsed: Duration);
}

/// A simple no-op progress reporter, used by tests and library users.
pub struct NoOpProgress;

impl Progress for NoOpProgress {
    fn stage(&self, _name: &str) {}
    fn stage_done(&self, _name: &str, _elapsed: Duration) {}
}

/// Writes compilation stage announcements to stderr.
pub struct SimpleProgress {
    quiet: bool,
    measure: bool,
}

impl SimpleProgress {
    pub fn new(quiet: bool, measure: bool) -> Self {
        Self { quiet, measure }
    }
}

impl Progress for SimpleProgress {
    fn stage(&self, name: &str) {
        if !self.quiet {
            eprintln!("→ {name}");
        }
    }
    fn stage_done(&self, name: &str, elapsed: Duration) {
        if !self.quiet && self.measure {
            eprintln!("  {name} took {}ms", elapsed.as_millis());
        }
    }
}
