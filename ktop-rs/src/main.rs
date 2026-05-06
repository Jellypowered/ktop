mod app;
mod config;
mod gpu;
mod system;
mod theme;
mod ui;

use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MIN_REFRESH_SECS: f64 = 0.25;
const INSTALL_SCRIPT_URL: &str =
    "https://raw.githubusercontent.com/brontoguana/ktop/master/install.sh";

fn main() {
    let mut refresh = 1.0f64;
    let mut sim = false;
    let mut theme_override: Option<String> = None;
    let mut no_alt_screen = env_flag("KTOP_NO_ALT_SCREEN");
    let mut no_sync = env_flag("KTOP_NO_SYNC");

    let args: Vec<String> = env::args().collect();
    if let Some(command) = args.get(1).map(String::as_str) {
        match command {
            "update" => {
                if args.get(2).is_some_and(|arg| is_help_arg(arg)) {
                    print_update_help();
                    return;
                }
                if let Err(e) = run_update() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            "uninstall" => {
                if args.get(2).is_some_and(|arg| is_help_arg(arg)) {
                    print_uninstall_help();
                    return;
                }
                if let Err(e) = run_uninstall(&args[0]) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            _ => {}
        }
    }

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--version" => {
                println!("ktop {}", VERSION);
                return;
            }
            "-h" | "--help" => {
                println!("ktop {} — system monitor for hybrid LLM workloads", VERSION);
                println!();
                println!("Usage: ktop [OPTIONS]");
                println!("       ktop <COMMAND>");
                println!();
                println!("Options:");
                println!("  -v, --version          Print version");
                println!("  -r, --refresh <SECS>   Refresh interval (default: 1.0)");
                println!("  --theme <NAME>         Color theme");
                println!("  --sim                  Simulation mode");
                println!("  --no-alt-screen        Draw in the current terminal screen");
                println!("  --no-sync              Disable synchronized terminal redraws");
                println!("  -h, --help             Print help");
                println!();
                println!("Commands:");
                println!("  update                 Run the official one-line update installer");
                println!("  uninstall              Remove this ktop executable");
                return;
            }
            "-r" | "--refresh" => {
                i += 1;
                if i < args.len() {
                    refresh = parse_refresh(&args[i]);
                }
            }
            "--theme" => {
                i += 1;
                if i < args.len() {
                    theme_override = Some(args[i].clone());
                }
            }
            "--sim" => {
                sim = true;
            }
            "--no-alt-screen" => {
                no_alt_screen = true;
            }
            "--no-sync" => {
                no_sync = true;
            }
            _ => {}
        }
        i += 1;
    }

    if let Err(e) = app::run(refresh, sim, theme_override, no_alt_screen, no_sync) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn parse_refresh(value: &str) -> f64 {
    value
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .map(|v| v.max(MIN_REFRESH_SECS))
        .unwrap_or(1.0)
}

fn env_flag(name: &str) -> bool {
    matches!(
        env::var(name).ok().as_deref(),
        Some("1")
            | Some("true")
            | Some("TRUE")
            | Some("yes")
            | Some("YES")
            | Some("on")
            | Some("ON")
    )
}

fn is_help_arg(arg: &str) -> bool {
    matches!(arg, "-h" | "--help")
}

fn print_update_help() {
    println!("ktop update");
    println!();
    println!("Runs the official one-line installer from the README:");
    println!("  curl -sSfL {} | bash", INSTALL_SCRIPT_URL);
}

fn print_uninstall_help() {
    println!("ktop uninstall");
    println!();
    println!("Removes only the ktop executable that was resolved from your PATH.");
    println!("User config and other system files are left untouched.");
}

fn run_update() -> Result<(), String> {
    println!("Running ktop update via the official installer...");
    println!("curl -sSfL {} | bash", INSTALL_SCRIPT_URL);

    let status = Command::new("bash")
        .arg("-c")
        .arg(format!("curl -sSfL {} | bash", INSTALL_SCRIPT_URL))
        .status()
        .map_err(|e| format!("failed to start update installer: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "update installer exited with status {}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ))
    }
}

fn run_uninstall(argv0: &str) -> Result<(), String> {
    let target = resolve_invoked_path(argv0)
        .map_err(|e| format!("could not resolve the ktop executable path: {}", e))?;

    validate_uninstall_target(&target)?;

    println!("Removing {}", target.display());
    match fs::remove_file(&target) {
        Ok(()) => {
            print_uninstall_success(&target);
            Ok(())
        }
        Err(e)
            if e.kind() == io::ErrorKind::PermissionDenied && is_readme_install_path(&target) =>
        {
            let status = Command::new("sudo")
                .arg("rm")
                .arg("-f")
                .arg(&target)
                .status()
                .map_err(|sudo_err| {
                    format!(
                        "permission denied removing {}; sudo failed to start: {}",
                        target.display(),
                        sudo_err
                    )
                })?;

            if status.success() {
                print_uninstall_success(&target);
                Ok(())
            } else {
                Err(format!(
                    "sudo rm exited with status {}",
                    status
                        .code()
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                ))
            }
        }
        Err(e) => Err(format!("failed to remove {}: {}", target.display(), e)),
    }
}

fn resolve_invoked_path(argv0: &str) -> io::Result<PathBuf> {
    let invoked = Path::new(argv0);
    if invoked.is_absolute() {
        return Ok(invoked.to_path_buf());
    }

    if argv0.contains('/') {
        return Ok(env::current_dir()?.join(invoked));
    }

    if let Some(path_var) = env::var_os("PATH") {
        let cwd = env::current_dir()?;
        for dir in env::split_paths(&path_var) {
            let absolute_dir = if dir.is_absolute() {
                dir
            } else {
                cwd.join(dir)
            };
            let candidate = absolute_dir.join(argv0);
            if is_executable_file(&candidate) {
                return Ok(candidate);
            }
        }
    }

    env::current_exe()
}

fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn validate_uninstall_target(path: &Path) -> Result<(), String> {
    if path.file_name().and_then(|name| name.to_str()) != Some("ktop") {
        return Err(format!(
            "refusing to remove {}; expected a file named ktop",
            path.display()
        ));
    }

    let metadata = fs::symlink_metadata(path)
        .map_err(|e| format!("could not inspect {}: {}", path.display(), e))?;
    if metadata.file_type().is_dir() {
        return Err(format!("refusing to remove directory {}", path.display()));
    }

    Ok(())
}

fn is_readme_install_path(path: &Path) -> bool {
    path == Path::new("/usr/local/bin/ktop")
}

fn print_uninstall_success(path: &Path) {
    println!("Removed {}", path.display());
    println!("Left ktop user config untouched.");
    println!("Run 'hash -r' if your shell still remembers the old command path.");
}
