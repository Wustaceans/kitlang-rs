use clap::{Parser, Subcommand};
use kitc_ffi::init_compiler_info;
use kitlang::codegen::frontend::Compiler;
use kitlang::error::CompilationError;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time;

type Error = Box<dyn std::error::Error>;

#[derive(Parser)]
#[command(name = "kitc", version, about = "Kit compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // TODO: the compiler artifacts should be deleted as soon as the final C program has been
    // successfully compiled. There should be a flag to disable this behavior.
    /// Compile a .kit file to an executable
    Compile {
        /// The `.kit` source file
        source: PathBuf,

        /// Add a source path (format: dir or dir:prefix). Can be repeated.
        #[arg(short = 'p', long = "source-path")]
        source_paths: Vec<String>,

        /// The libraries to link against
        #[arg(short, long)]
        libs: Vec<String>,

        /// Additional C compiler flags (e.g. "-O2 -g").
        /// Translated to the target toolchain; output stays C99-compatible.
        #[arg(long)]
        cflags: Vec<String>,

        /// Additional library search paths (can be repeated).
        #[arg(long = "lib-path")]
        lib_paths: Vec<String>,

        /// Override the C compiler executable to invoke.
        #[arg(long = "cc")]
        cc: Option<String>,

        /// Compile and immediately run the executable
        #[arg(long)]
        run: bool,

        /// Print compilation timing information
        #[arg(long)]
        measure: bool,
    },
}

fn main() -> Result<(), Error> {
    env_logger::init();

    // Initialize compiler detection early to fail fast if no C compiler is available
    if init_compiler_info().is_none() {
        eprintln!(
            "No C compiler found. Please install gcc, clang, or MSVC and ensure it's in PATH."
        );
        eprintln!("You can also set KITC_CC to specify a compiler explicitly.");
        std::process::exit(1);
    }

    // Destructure the Cli to get the `command` field
    let Cli { command } = Cli::parse();

    match command {
        Commands::Compile {
            source,
            source_paths,
            libs,
            cflags,
            lib_paths,
            cc,
            run,
            measure,
        } => {
            if !source.exists() {
                eprintln!(
                    "{} does not exist. Please check again the path and try again.",
                    source.display()
                );
                return Ok(());
            }

            let exe_path = match compile(
                &source,
                &source_paths,
                &libs,
                &cflags,
                &lib_paths,
                cc.as_deref(),
                measure,
            ) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("{}", e.render());
                    std::process::exit(1);
                }
            };
            if run {
                run_executable(&exe_path)?;
            } else {
                println!("→ Successfully compiled!");
            }
        }
    }
    Ok(())
}

fn compile(
    source: &Path,
    source_paths: &[String],
    libs: &[String],
    cflags: &[String],
    lib_paths: &[String],
    cc: Option<&str>,
    measure: bool,
) -> Result<PathBuf, CompilationError> {
    let init = time::Instant::now();

    let ext = if cfg!(windows) { "exe" } else { "" };
    let mut exe_path = source.with_extension(ext);

    // Canonicalize so Command::new can find it even when the path has no directory separator
    // (otherwise Command::new searches PATH instead of the current directory).
    if exe_path.is_relative()
        && let Ok(cwd) = std::env::current_dir()
    {
        exe_path = cwd.join(&exe_path);
    }

    let mut compiler = Compiler::new(
        vec![source.to_path_buf()],
        &exe_path,
        libs.to_vec(),
        source_paths,
        cflags.to_vec(),
        lib_paths.to_vec(),
        cc.map(PathBuf::from),
    );

    compiler.compile()?;

    if measure {
        println!("→ Compiled in {}ms", init.elapsed().as_millis());
    }

    Ok(exe_path)
}

// TODO: return the exit status from the compiler code, and return Err() if it failed, probably
// adding an exit status (to exit with).
fn run_executable(exe_path: &Path) -> Result<(), String> {
    let status = Command::new(exe_path)
        .status()
        .map_err(|e| format!("failed to launch executable: {e}"))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
