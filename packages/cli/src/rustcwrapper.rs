use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::{args, vars},
    path::PathBuf,
    process::ExitCode,
};

/// The environment variable indicating where the args file is located.
///
/// When `dx-rustc` runs, it writes its arguments to this file.
pub const DX_RUSTC_WRAPPER_ENV_VAR: &str = "DX_RUSTC";

/// The environment variable indicating the directory where workspace crate args are stored.
///
/// When `dx` is used as `RUSTC_WORKSPACE_WRAPPER`, each workspace crate's args are saved
/// to a separate file in this directory, keyed by crate name.
pub const DX_RUSTC_WORKSPACE_WRAPPER_ENV_VAR: &str = "DX_RUSTC_WORKSPACE";

/// A map of crate names to their rustc args for workspace crates.
pub type WorkspaceRustcArgs = HashMap<String, RustcArgs>;

/// Is `dx` being used as a rustc wrapper?
///
/// This is primarily used to intercept cargo, enabling fast hot-patching by caching the environment
/// cargo setups up for the user's current project.
///
/// In a different world we could simply rely on cargo printing link args and the rustc command, but
/// it doesn't seem to output that in a reliable, parseable, cross-platform format (ie using command
/// files on windows...), so we're forced to do this interception nonsense.
pub fn is_wrapping_rustc() -> bool {
    std::env::var(DX_RUSTC_WRAPPER_ENV_VAR).is_ok()
        || std::env::var(DX_RUSTC_WORKSPACE_WRAPPER_ENV_VAR).is_ok()
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RustcArgs {
    pub args: Vec<String>,
    pub envs: Vec<(String, String)>,
    /// it doesn't include first program name argument
    pub link_args: Vec<String>,
}

/// Check if the arguments indicate a linking step, including those in command files.
fn has_linking_args() -> bool {
    for arg in std::env::args() {
        // Direct check for linker-like arguments
        if arg.ends_with(".o") || arg == "-flavor" {
            return true;
        }

        // Check inside command files
        if let Some(path_str) = arg.strip_prefix('@') {
            if let Ok(file_binary) = std::fs::read(path_str) {
                // Handle both UTF-8 and UTF-16LE encodings for response files.
                let content = String::from_utf8(file_binary.clone()).unwrap_or_else(|_| {
                    let binary_u16le: Vec<u16> = file_binary
                        .chunks_exact(2)
                        .map(|a| u16::from_le_bytes([a[0], a[1]]))
                        .collect();
                    String::from_utf16_lossy(&binary_u16le)
                });

                // Check if any line in the command file contains linking indicators.
                if content.lines().any(|line| {
                    let trimmed_line = line.trim().trim_matches('"');
                    trimmed_line.ends_with(".o") || trimmed_line == "-flavor"
                }) {
                    return true;
                }
            }
        }
    }

    false
}

/// Run rustc directly, but output the result to a file.
///
/// <https://doc.rust-lang.org/cargo/reference/config.html#buildrustc>
pub fn run_rustc() -> ExitCode {
    // If we are being asked to link, delegate to the linker action.
    if has_linking_args() {
        return crate::link::LinkAction::from_env()
            .expect("Linker action not found")
            .run_link();
    }

    // Cargo invokes a wrapper like: `wrapper-name rustc [args...]`
    // We skip our own executable name (`wrapper-name`) to get the args passed to us.
    let captured_args = args().skip(1).collect::<Vec<_>>();

    let rustc_args = RustcArgs {
        args: captured_args.clone(),
        envs: vars().collect::<_>(),
        link_args: Default::default(),
    };

    // Extract crate name and crate type from args - we'll use these to store per-crate rustc args
    let crate_name = rustc_args
        .args
        .iter()
        .skip_while(|arg| *arg != "--crate-name")
        .nth(1)
        .cloned();

    let crate_type = rustc_args
        .args
        .iter()
        .skip_while(|arg| *arg != "--crate-type")
        .nth(1)
        .cloned()
        .unwrap_or_else(|| "lib".to_string());

    // Only cache args if we have a valid crate name (not ___ which is used for fresh builds)
    if let Some(ref name) = crate_name {
        if name != "___" {
            // Check if we're running as RUSTC_WORKSPACE_WRAPPER (for all workspace crates)
            // or RUSTC_WRAPPER (for the top-level crate only)
            if let Ok(workspace_dir) = std::env::var(DX_RUSTC_WORKSPACE_WRAPPER_ENV_VAR) {
                // Store args in a per-crate file within the workspace args directory
                // Include crate type in filename to distinguish lib/bin with same name
                let workspace_dir = PathBuf::from(workspace_dir);
                std::fs::create_dir_all(&workspace_dir)
                    .expect("Failed to create workspace args directory");

                let crate_args_file = workspace_dir.join(format!("{}_{}.json", name, crate_type));
                let serialized_args =
                    serde_json::to_string(&rustc_args).expect("Failed to serialize rustc args");
                std::fs::write(&crate_args_file, serialized_args)
                    .expect("Failed to write workspace crate rustc args to file");
            } else if let Ok(var_file) = std::env::var(DX_RUSTC_WRAPPER_ENV_VAR) {
                // Legacy behavior: store in a single file (for the top-level crate)
                let var_file = PathBuf::from(var_file);
                let parent_dir = var_file
                    .parent()
                    .expect("Args file path has no parent directory");
                std::fs::create_dir_all(parent_dir)
                    .expect("Failed to create parent directory for args file");

                let serialized_args =
                    serde_json::to_string(&rustc_args).expect("Failed to serialize rustc args");
                std::fs::write(&var_file, serialized_args)
                    .expect("Failed to write rustc args to file");
            }
        }
    }

    // Run the actual rustc command.
    // We want all stdout/stderr to be inherited, so the user sees the compiler output.
    let mut cmd = std::process::Command::new("rustc");

    // The first argument in `captured_args` is "rustc", which we need to skip
    // when passing arguments to the `rustc` command we are spawning.
    cmd.args(captured_args.iter().skip(1));
    cmd.envs(rustc_args.envs);
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    cmd.current_dir(std::env::current_dir().expect("Failed to get current dir"));

    // Spawn the process and propagate its exit code.
    let status = cmd.status().expect("Failed to execute rustc command");
    std::process::exit(status.code().unwrap_or(1)); // Exit with 1 if process was killed by signal
}

/// Load workspace rustc args from the workspace args directory.
/// Returns a map of crate name -> RustcArgs for all workspace crates.
pub fn load_workspace_rustc_args(workspace_args_dir: &std::path::Path) -> WorkspaceRustcArgs {
    let mut result = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(workspace_args_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(crate_name) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        if let Ok(args) = serde_json::from_str::<RustcArgs>(&contents) {
                            result.insert(crate_name.to_string(), args);
                        }
                    }
                }
            }
        }
    }

    result
}
