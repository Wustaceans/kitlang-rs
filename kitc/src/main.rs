use clap::{Parser, Subcommand};
use kitc_ffi::init_compiler_info;
use kitlang::codegen::SimpleProgress;
use kitlang::codegen::frontend::Compiler;
use kitlang::error::CompilationError;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time;

type Error = Box<dyn std::error::Error>;

struct CompileOpts<'cfg> {
    source: &'cfg Path,
    source_paths: &'cfg [String],
    libs: &'cfg [String],
    cflags: &'cfg [String],
    lib_paths: &'cfg [String],
    cc: Option<&'cfg str>,
    measure: bool,
    quiet: bool,
}

#[derive(Parser)]
#[command(name = "kitc", version, about = "Kit compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        ///
        /// The flags are translated to the target toolchain, but the output remains C99-compatible.
        #[arg(long)]
        cflags: Vec<String>,

        /// Additional library search paths (can be repeated).
        #[arg(long = "lib-path")]
        lib_paths: Vec<String>,

        /// Override the C compiler executable to invoke.
        #[arg(long)]
        cc: Option<String>,

        /// Compile and immediately run the executable.
        ///
        /// If the executable exits with a failure code (!= 0), `kitc` exits with the same code.
        #[arg(long)]
        run: bool,

        /// Print compilation timing information
        #[arg(long)]
        measure: bool,

        /// Suppress progress output
        #[arg(long)]
        quiet: bool,
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
            quiet,
        } => {
            if !source.exists() {
                eprintln!(
                    "{} does not exist. Please check again the path and try again.",
                    source.display()
                );
                return Ok(());
            }

            let exe_path = match compile(CompileOpts {
                source: &source,
                source_paths: &source_paths,
                libs: &libs,
                cflags: &cflags,
                lib_paths: &lib_paths,
                cc: cc.as_deref(),
                measure,
                quiet,
            }) {
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

fn compile(opts: CompileOpts<'_>) -> Result<PathBuf, CompilationError> {
    let CompileOpts {
        source,
        source_paths,
        libs,
        cflags,
        lib_paths,
        cc,
        measure,
        quiet,
    } = opts;

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

    let progress = SimpleProgress::new(quiet, measure);
    compiler.compile(&progress)?;

    if measure {
        eprintln!("→ Compiled in {}ms", init.elapsed().as_millis());
    }

    Ok(exe_path)
}

fn run_executable(exe_path: &Path) -> Result<(), String> {
    let status = Command::new(exe_path)
        .status()
        .map_err(|e| format!("failed to launch executable: {e}"))?;

    if !status.success() {
        let code = status.code();

        if let Some(exit_code) = code {
            eprintln!("'{}' exited with code: {}", exe_path.display(), exit_code);
        }

        std::process::exit(code.unwrap_or(1));
    }
    Ok(())
}
