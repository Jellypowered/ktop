# Changelog

## 1.0.16 — 2026-05-06

- Made Basic color mode avoid all extended foreground color SGR output, including indexed `38;5;n` sequences as well as RGB `38;2;r;g;b`
- Clear/reset the terminal when toggling color mode so stale truecolor-styled cells are repainted immediately
- Kept truecolor as the default; Basic is now the safest monochrome fallback for terminals that flash on extended foreground color escapes
- Tested: `cargo check`, debug build, PTY capture with Basic mode showing zero RGB foreground and zero indexed foreground SGR sequences

## 1.0.15 — 2026-05-06

- Added a main-screen `c Color` toggle that switches between truecolor and basic ANSI color output
- Persisted color mode in the existing user config alongside the selected theme, defaulting existing installs to truecolor
- Mapped RGB and indexed theme colors to ANSI/basic colors when Basic color mode is active
- Tested: `cargo check`, `cargo build --release`, PTY smoke for persisted `c` toggle and RGB-SGR removal in Basic mode

## 1.0.14 — 2026-05-06

- Buffered terminal output before handing stdout to ratatui so frame flushes are less fragmented over remote transports
- Simplified `--compat` progress bars to one filled span plus one empty span instead of per-cell RGB gradient spans, reducing paint-stream volume during each poll

## 1.0.13 — 2026-05-06

- Made `--compat` avoid dark-gray ANSI styling and shaded filler glyphs, which can animate or flash in some remote terminals even after ktop exits
- Compatibility mode now clears the current screen on entry and exit so stale styled cells do not remain behind the shell prompt
- Reset terminal attributes and colors during cleanup before returning control to the shell

## 1.0.12 — 2026-05-06

- Reduced lower-screen repaint churn by smoothing process CPU percentages before sorting/display
- Added deterministic process-table tie breakers so similar samples do not reshuffle rows unnecessarily
- Made `--compat` use low-churn rendering: moving sparklines are suppressed and process tables refresh every 10s
- Intended for terminals that visibly flash only the cells/regions rewritten by live metric updates

## 1.0.11 — 2026-05-06

- Added `ktop diagnose-render` to isolate visual flashing by terminal escape category
- Render diagnostics cover plain output, cursor repaint, ANSI color, 24-bit RGB color, Unicode blocks, full-frame painting, repeated clear, alternate screen, and synchronized update
- Intended follow-up for terminals where size diagnostics are stable but the TUI still visibly flashes

## 1.0.10 — 2026-05-06

- Avoid repeated terminal clears from unstable resize reporting by using a fixed viewport and debounced resize handling
- Added `ktop diagnose-terminal` to report terminal size jitter and resize event storms
- Added `--compat` / `KTOP_COMPAT=1` for terminals that still flash during normal full-screen rendering
- Compatibility mode disables alternate screen and synchronized redraws, skips startup clears, and defaults to a 2s refresh unless `-r` is set
- Unknown commands/options now exit with a usage error instead of falling through into the TUI
- Terminal diagnostics still print size samples when stdin event polling is unavailable

## 1.0.9 — 2026-05-06

- Added `ktop update` to run the official README one-line update installer
- Added `ktop uninstall` to remove only the resolved `ktop` executable while leaving user config and unrelated system files untouched
- Added command help for update/uninstall
- Tested: `cargo check`, `cargo build --release`, command help smoke tests, temp-copy uninstall smoke test

## 1.0.8 — 2026-05-06

- Reduced repaint flashing on remote/web terminals by wrapping TUI draws in synchronized terminal updates
- Hide the cursor while ktop is running and restore it on exit
- Added `--no-alt-screen` / `KTOP_NO_ALT_SCREEN=1` for terminals whose alternate-screen support flashes or tears
- Added `--no-sync` / `KTOP_NO_SYNC=1` as a fallback for terminals that mishandle synchronized update escape sequences
- Clamp refresh intervals below 0.25s to avoid accidental tight redraw loops
- Cap footer right-side status width to the visible terminal width on narrow terminals
- Tested: `cargo check`, `cargo build --release`

## 1.0.7 — 2026-04-12

- Keep the footer power slot visible on narrower terminals by right-anchoring power and OOM status
- Let the left-side help text truncate first instead of clipping PWR off the screen
- Tested: `cargo build --release`

## 1.0.6 — 2026-04-12

- Always show the footer power slot
- Display `PWR n/a` when the host exposes no usable power telemetry instead of hiding the field entirely
- Use saturating footer padding so the power segment is not squeezed out on narrower terminals
- Tested: `cargo build --release`

## 1.0.5 — 2026-04-12

- Add footer power estimate segment before OOM status when live sensors are available
- Estimate uses CPU package power from Linux powercap/hwmon plus NVIDIA NVML and AMD hwmon GPU power
- Hide the power field when the host exposes no usable power sensors instead of showing a fake value
- Tested: `cargo build --release`

## 1.0.4 — 2026-04-12

- Switch from musl static linking to glibc dynamic linking for NVIDIA GPU compatibility
- musl binaries cannot `dlopen` glibc-linked shared libraries such as `libnvidia-ml.so`, which caused NVIDIA GPU detection to fail
- CI now uses `cross` to build GNU-targeted binaries against an older glibc for portability
- Install script updated to fetch GNU-targeted binaries
- Tested: local build, version check, `ldd` confirms dynamic linking

## 1.0.3 — 2026-03-22

- Static linking via musl — binary now runs on any Linux distro regardless of GLIBC version
- No more "GLIBC_2.xx not found" errors on older systems
- Install script updated for new musl-linked binaries

## 1.0.2 — 2026-03-18

- Show total CPU percentage across all cores (e.g. 400% = 4 cores maxed)
- Show available RAM and disk cache size under memory section

## 1.0.0 — 2026-03-10

- Complete rewrite from Python to Rust — single static binary (~1.2 MB)
- Near-zero CPU and memory overhead (2-5 MB RAM vs 30-50 MB for Python version)
- Instant startup, no runtime dependencies
- One-line install script with automatic upgrades
- GitHub Actions release workflow for x86_64 and aarch64 binaries
- All features from 0.9.0 preserved: 50 themes, NVIDIA + AMD GPU monitoring, sparklines, OOM tracking, process tables, temperature strip

## 0.9.0 — 2026-02-11

- **AMD GPU support** via Linux sysfs — no new dependencies required
- AMD GPUs detected automatically from `/sys/class/drm/card*/device/vendor` (vendor `0x1002`)
- GPU utilization from `gpu_busy_percent`, VRAM from `mem_info_vram_total`/`mem_info_vram_used`
- AMD GPU temperatures from hwmon `temp1_input`/`temp1_crit`
- Mixed NVIDIA+AMD systems show all GPUs together with unified numbering
- Gracefully handles missing sysfs files (older cards, APUs): util→0%, VRAM→0/0 GB, temp→N/A
- GPU name from `product_name` with fallback to PCI device ID
- Strips "AMD " and "Advanced Micro Devices, Inc. " prefixes in GPU panel subtitles
- NVIDIA-only path unchanged — refactored `gpu_ok` → `nvidia_ok` for vendor-specific guards
- Tested: verified no regression on NVIDIA-only system, ran `ktop --sim`, reinstalled via setup.sh

## 0.8.1 — 2026-02-11

- OOM tracker now detects `systemd-oomd` kills in addition to kernel OOM kills
- Fixed `capture_output=True` + `stderr=DEVNULL` conflict that silently broke OOM detection entirely
- Uses `short-unix` journal output for reliable timestamp comparison between OOM sources
- Scope names cleaned up: strips `.scope` suffix and UUIDs for readable display (e.g. `tmux-spawn`)
- Added `__version__` and `ktop --version` flag
- Tested: verified both kernel OOM and systemd-oomd kills detected from real journal entries, reinstalled via setup.sh

## 0.8.0 — 2026-02-10

- CPU process table now shows both Core % (per-core, matches `top`) and CPU % (system-wide)
- Fixed per-process CPU calculation: no longer divides by num_cpus
- Raw binary I/O for `/proc` reads: `os.open`/`os.read`/`os.close` instead of Python file objects
- Binary mode (`rb`) and `split(None, 22)` to avoid decoding overhead and unnecessary allocations
- CPU frequency read from sysfs (`scaling_cur_freq`) instead of `psutil.cpu_freq()` (9ms → 0.02ms), with psutil fallback
- Deferred `/proc/pid/statm` reads: only read for the top 20 displayed processes instead of all ~1690
- `cpu_count` cached at init; `cpu_freq` polled every 5s
- Process CPU baselines seeded at startup for accurate first-frame deltas
- Process tables populate after 1s, refresh every 3s
- Total frame time: 228ms → 20ms (11x improvement)
- Tested: profiled with `--sim`, reinstalled via setup.sh

## 0.7.0 — 2026-02-10

- Process scanning optimized: replaced `psutil.process_iter` with direct `/proc/pid/stat` + `/proc/pid/statm` reads (214ms → 23ms per scan, ~9x faster)
- Process list cached for 5 seconds instead of rescanning every frame
- Added `--sim` flag for simulation mode (fake OOM kills, profiling output to `/tmp/ktop_profile.log`)
- Profiler logs avg/max/calls per section every 5s in sim mode
- OOM kill tracker uses `journalctl` for persistent 8-hour lookback instead of `dmesg` kernel ring buffer
- OOM status shows solid block `█` when OOM detected, hollow `░` when clear
- Tested: profiled with `--sim`, reinstalled via setup.sh

## 0.6.0 — 2026-02-10

- Network panel: sparklines now centered between bar charts — upload sparkline extends upward, download sparkline extends downward using upper-block Unicode characters
- Added `SPARK_DOWN` character set and `_sparkline_down()` for top-down sparklines
- Network upload and download now have separate theme colors (`net_up` defaults to GPU color, `net_down` defaults to net color)
- Theme picker swatches updated to show net_up/net_down colors
- Status bar now shows most recent OOM kill (process name + timestamp) on the right side
- Temperature strip between charts and process tables with hardware-accurate thresholds (GPU slowdown from NVML, CPU critical from psutil, 85°C JEDEC for memory)
- Temperature strip border uses theme `bar_mid` color; entries evenly spaced
- GPU bar charts now use dynamic width matching CPU/memory panel sizing
- Tested: reinstalled via setup.sh

## 0.5.0 — 2026-02-10

- Added Network panel to the second row (layout is now Network, CPU, Memory)
- Shows upload and download bar charts with auto-scaling and sparkline history
- Displays current speed (B/s, KB/s, MB/s, GB/s) and peak observed speed
- Added `net` color to theme system (defaults to CPU color for all existing themes)
- Theme picker preview and swatches updated to include network color
- Added spacing between GPU utilization sparkline and memory bar chart
- Tested: reinstalled via setup.sh

## 0.4.3 — 2026-02-10

- Memory values shown in GB or MB with max 1 decimal place
- Tested: reinstalled via setup.sh

## 0.4.2 — 2026-02-10

- Bar charts now render a smooth per-block gradient from bar_low to bar_high across the full 0-100% width
- Each filled block gets its own interpolated hex color via linear RGB lerp
- RGB conversion cached to avoid re-parsing color names every frame
- Tested: reinstalled via setup.sh

## 0.4.1 — 2026-02-10

- Memory process table now shows Used (RSS−shared) + Shared columns instead of just RSS
- Uses `memory_info().shared` from `/proc/statm` (instant) instead of `memory_full_info()` which reads `/proc/smaps` and was extremely slow on systems with large memory maps, causing ktop to hang on launch
- Tested: reinstalled via setup.sh, launches instantly

## 0.4.0 — 2026-02-10

- Fixed arrow key input: rewrote `_read_key` to use `os.read()` on raw fd instead of buffered `sys.stdin.read()`, so escape sequences are captured atomically
- Responsive input loop: keys polled at 50ms with immediate redraw on keypress
- Theme picker: color swatches (background-colored chips for gpu/cpu/mem/bar) right-aligned next to each theme name with gaps between colors
- Sparklines aligned with bar chart left edge in GPU and CPU panels
- Bar charts now render as gradients: green→yellow→red across the filled region (thresholds at 50% and 80%)
- Tested: reinstalled via setup.sh

## 0.3.1 — 2026-02-10

- Sparklines now match the color of their metric (GPU util, GPU mem, CPU) instead of default white
- Sparklines dynamically fill the width of their enclosing panel (with margin) instead of fixed 20 chars
- Increased history buffer from 60 to 300 samples to support wide terminals
- Tested: reinstalled via setup.sh

## 0.3.0 — 2026-02-10

- Added theme system with 50 color themes (press `t` to open theme picker)
- Arrow keys + Enter to select theme, ESC to cancel
- Theme preference saved to `~/.config/ktop/config.json` and persists across sessions
- `--theme` CLI flag to set theme from command line
- Bottom status bar showing keybindings (q/ESC quit, t themes)
- Proper arrow key handling for theme picker navigation
- Tested: both main view and theme picker render correctly; config persistence works

## 0.2.0 — 2026-02-10

- GPU panels now laid out horizontally — all GPUs visible side by side
- Added q and ESC keys to quit the app (in addition to Ctrl+C)
- Tested: horizontal layout renders correctly with 3 GPUs at 140-col width

## 0.1.1 — 2026-02-10

- Added `setup.sh` installer script — installs `ktop` as a command to `~/.local/bin` (or `/usr/local/bin` with `--system`)
- Suppressed pynvml FutureWarning deprecation noise
- Updated README with quick-install instructions
- Tested: `ktop` command works system-wide after `./setup.sh`

## 0.1.0 — 2026-02-10

- Initial release of ktop
- GPU utilization and memory monitoring with per-GPU sparkline history (NVIDIA)
- CPU usage monitoring with overall bar chart and sparkline history
- RAM and swap usage bar charts
- Top 10 processes by memory usage table
- Top 10 processes by CPU usage table
- Color-coded thresholds (green/yellow/red)
- Configurable refresh rate via `-r` flag
- Tested: renders correctly with 3x NVIDIA RTX 2000 Ada GPUs, 128-core CPU, ~1 TB RAM
