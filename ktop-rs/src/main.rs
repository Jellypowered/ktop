mod app;
mod config;
mod gpu;
mod system;
mod theme;
mod ui;

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, IsTerminal};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use crossterm::terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MIN_REFRESH_SECS: f64 = 0.25;
const COMPAT_REFRESH_SECS: f64 = 2.0;
const DIAGNOSTIC_DURATION_SECS: f64 = 5.0;
const INSTALL_SCRIPT_URL: &str =
    "https://raw.githubusercontent.com/brontoguana/ktop/master/install.sh";

fn main() {
    let mut refresh = 1.0f64;
    let mut refresh_explicit = false;
    let mut sim = false;
    let mut theme_override: Option<String> = None;
    let mut no_alt_screen = env_flag("KTOP_NO_ALT_SCREEN");
    let mut no_sync = env_flag("KTOP_NO_SYNC");
    let mut compat = env_flag("KTOP_COMPAT");

    let args: Vec<String> = env::args().collect();
    if let Some(command) = args.get(1).map(String::as_str) {
        match command {
            "update" => {
                if let Some(arg) = args.get(2) {
                    if is_help_arg(arg) {
                        print_update_help();
                        return;
                    }
                    exit_usage_error(format!("unexpected argument '{}' for ktop update", arg));
                }
                if let Err(e) = run_update() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            "uninstall" => {
                if let Some(arg) = args.get(2) {
                    if is_help_arg(arg) {
                        print_uninstall_help();
                        return;
                    }
                    exit_usage_error(format!(
                        "unexpected argument '{}' for ktop uninstall",
                        arg
                    ));
                }
                if let Err(e) = run_uninstall(&args[0]) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            "diagnose-terminal" => {
                if let Err(e) = run_terminal_diagnostics(&args[2..]) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            _ if command.starts_with('-') => {}
            _ => exit_usage_error(format!("unknown command '{}'", command)),
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
                print_main_help();
                return;
            }
            "-r" | "--refresh" => {
                i += 1;
                if i < args.len() {
                    refresh = parse_refresh(&args[i]);
                    refresh_explicit = true;
                } else {
                    exit_usage_error("missing value for --refresh");
                }
            }
            "--theme" => {
                i += 1;
                if i < args.len() {
                    theme_override = Some(args[i].clone());
                } else {
                    exit_usage_error("missing value for --theme");
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
            "--compat" => {
                compat = true;
            }
            other if other.starts_with('-') => {
                exit_usage_error(format!("unknown option '{}'", other));
            }
            other => exit_usage_error(format!("unexpected argument '{}'", other)),
        }
        i += 1;
    }

    if compat {
        no_alt_screen = true;
        no_sync = true;
        if !refresh_explicit {
            refresh = COMPAT_REFRESH_SECS;
        }
    }

    if let Err(e) = app::run(
        refresh,
        sim,
        theme_override,
        no_alt_screen,
        no_sync,
        compat,
    ) {
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

fn parse_duration_secs(value: &str) -> f64 {
    value
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| v.clamp(1.0, 60.0))
        .unwrap_or(DIAGNOSTIC_DURATION_SECS)
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

fn print_main_help() {
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
    println!("  --compat               Use conservative remote-terminal drawing");
    println!("  -h, --help             Print help");
    println!();
    println!("Commands:");
    println!("  diagnose-terminal      Report terminal size and resize behavior");
    println!("  update                 Run the official one-line update installer");
    println!("  uninstall              Remove this ktop executable");
}

fn exit_usage_error(message: impl AsRef<str>) -> ! {
    eprintln!("Error: {}", message.as_ref());
    eprintln!("Run 'ktop --help' for usage.");
    std::process::exit(2);
}

fn print_update_help() {
    println!("ktop update");
    println!();
    println!("Runs the official one-line installer from the README:");
    println!("  curl -sSfL {} | bash", INSTALL_SCRIPT_URL);
}

fn print_diagnostics_help() {
    println!("ktop diagnose-terminal");
    println!();
    println!("Reports terminal size stability and resize events without starting the TUI.");
    println!();
    println!("Usage: ktop diagnose-terminal [--duration SECS]");
}

fn run_terminal_diagnostics(args: &[String]) -> Result<(), String> {
    let mut duration = DIAGNOSTIC_DURATION_SECS;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_diagnostics_help();
                return Ok(());
            }
            "--duration" => {
                i += 1;
                if let Some(value) = args.get(i) {
                    duration = parse_duration_secs(value);
                } else {
                    return Err("missing value for --duration".to_string());
                }
            }
            other => return Err(format!("unknown diagnose-terminal option '{}'", other)),
        }
        i += 1;
    }

    let initial_size = terminal::size()
        .map_err(|e| format!("could not read terminal size: {}", e))?;
    let mut stats = TerminalDiagnostics {
        duration,
        stdin_tty: io::stdin().is_terminal(),
        stdout_tty: io::stdout().is_terminal(),
        stderr_tty: io::stderr().is_terminal(),
        initial_size,
        min_width: initial_size.0,
        max_width: initial_size.0,
        min_height: initial_size.1,
        max_height: initial_size.1,
        ..TerminalDiagnostics::default()
    };

    let raw_mode = stats.stdin_tty && terminal::enable_raw_mode().is_ok();
    let started = Instant::now();
    let deadline = started + Duration::from_secs_f64(duration);

    let collect_result = (|| -> Result<(), String> {
        while Instant::now() < deadline {
            let size = terminal::size()
                .map_err(|e| format!("could not read terminal size during probe: {}", e))?;
            stats.record_size(size);

            if raw_mode {
                while event::poll(Duration::from_millis(0))
                    .map_err(|e| format!("could not poll terminal events: {}", e))?
                {
                    match event::read()
                        .map_err(|e| format!("could not read terminal event: {}", e))?
                    {
                        Event::Resize(width, height) => {
                            stats.resize_events += 1;
                            stats.record_event_size((width, height));
                        }
                        Event::Key(_) => stats.key_events += 1,
                        Event::Mouse(_) => stats.mouse_events += 1,
                        Event::Paste(_) => stats.paste_events += 1,
                        Event::FocusGained | Event::FocusLost => stats.focus_events += 1,
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    })();

    if raw_mode {
        let _ = terminal::disable_raw_mode();
    }
    collect_result?;

    stats.print(raw_mode);
    Ok(())
}

#[derive(Default)]
struct TerminalDiagnostics {
    duration: f64,
    stdin_tty: bool,
    stdout_tty: bool,
    stderr_tty: bool,
    initial_size: (u16, u16),
    samples: u64,
    size_changes: u64,
    resize_events: u64,
    key_events: u64,
    mouse_events: u64,
    focus_events: u64,
    paste_events: u64,
    min_width: u16,
    max_width: u16,
    min_height: u16,
    max_height: u16,
    last_size: Option<(u16, u16)>,
    sampled_sizes: BTreeMap<(u16, u16), u64>,
    resize_event_sizes: BTreeMap<(u16, u16), u64>,
}

impl TerminalDiagnostics {
    fn record_size(&mut self, size: (u16, u16)) {
        self.samples += 1;
        if self.last_size.is_some_and(|last| last != size) {
            self.size_changes += 1;
        }
        self.last_size = Some(size);
        self.min_width = self.min_width.min(size.0);
        self.max_width = self.max_width.max(size.0);
        self.min_height = self.min_height.min(size.1);
        self.max_height = self.max_height.max(size.1);
        *self.sampled_sizes.entry(size).or_insert(0) += 1;
    }

    fn record_event_size(&mut self, size: (u16, u16)) {
        *self.resize_event_sizes.entry(size).or_insert(0) += 1;
    }

    fn print(&self, raw_mode: bool) {
        println!("ktop terminal diagnostics");
        println!("version: {}", VERSION);
        println!("duration_secs: {:.1}", self.duration);
        println!("stdin_tty: {}", self.stdin_tty);
        println!("stdout_tty: {}", self.stdout_tty);
        println!("stderr_tty: {}", self.stderr_tty);
        println!("raw_mode_probe: {}", raw_mode);
        println!("TERM: {}", env::var("TERM").unwrap_or_else(|_| "(unset)".to_string()));
        println!(
            "COLORTERM: {}",
            env::var("COLORTERM").unwrap_or_else(|_| "(unset)".to_string())
        );
        println!("initial_size: {}x{}", self.initial_size.0, self.initial_size.1);
        println!("samples: {}", self.samples);
        println!("sampled_size_changes: {}", self.size_changes);
        println!("sampled_unique_sizes: {}", self.sampled_sizes.len());
        println!(
            "sampled_width_range: {}..{}",
            self.min_width, self.max_width
        );
        println!(
            "sampled_height_range: {}..{}",
            self.min_height, self.max_height
        );
        println!("resize_events: {}", self.resize_events);
        println!("key_events: {}", self.key_events);
        println!("mouse_events: {}", self.mouse_events);
        println!("focus_events: {}", self.focus_events);
        println!("paste_events: {}", self.paste_events);
        println!("sampled_sizes: {}", format_size_counts(&self.sampled_sizes));
        println!(
            "resize_event_sizes: {}",
            format_size_counts(&self.resize_event_sizes)
        );

        let unstable_size = self.size_changes > 1 || self.sampled_sizes.len() > 1;
        let resize_storm = self.resize_events > 2;
        println!("unstable_size: {}", unstable_size);
        println!("resize_event_storm: {}", resize_storm);
        if unstable_size || resize_storm {
            println!(
                "diagnosis: terminal size is changing during the probe; older full-screen rendering can repeatedly clear in this condition"
            );
        } else {
            println!(
                "diagnosis: terminal size stayed stable during the probe; flashing is likely caused by a specific draw escape or terminal repaint behavior"
            );
        }
    }
}

fn format_size_counts(sizes: &BTreeMap<(u16, u16), u64>) -> String {
    if sizes.is_empty() {
        return "(none)".to_string();
    }

    sizes
        .iter()
        .map(|((width, height), count)| format!("{}x{}={}", width, height, count))
        .collect::<Vec<_>>()
        .join(", ")
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
