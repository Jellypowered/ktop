use crate::app::AppState;
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;
use std::collections::VecDeque;

const SPARK: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const SPARK_DOWN: &[char] = &[
    ' ',
    '▔',
    '\u{1FB82}',
    '\u{1FB83}',
    '▀',
    '\u{1FB84}',
    '\u{1FB85}',
    '\u{1FB86}',
    '█',
];

// ── Sparkline helpers ──

fn sparkline(values: &VecDeque<f64>, width: usize) -> String {
    if values.is_empty() {
        return String::new();
    }
    let vals: Vec<f64> = if values.len() > width {
        values.iter().skip(values.len() - width).copied().collect()
    } else {
        values.iter().copied().collect()
    };
    let mut out = String::new();
    for v in &vals {
        let v = v.clamp(0.0, 100.0);
        let idx = if v <= 5.0 {
            0
        } else {
            let i = 1 + ((v - 5.0) / 95.0 * (SPARK.len() as f64 - 2.0)) as usize;
            i.min(SPARK.len() - 1)
        };
        out.push(SPARK[idx]);
    }
    out
}

fn sparkline_double(values: &VecDeque<f64>, width: usize) -> (String, String) {
    if values.is_empty() {
        return (String::new(), String::new());
    }
    let vals: Vec<f64> = if values.len() > width {
        values.iter().skip(values.len() - width).copied().collect()
    } else {
        values.iter().copied().collect()
    };
    let n = SPARK.len() - 1; // 8 levels per row
    let mut top = String::new();
    let mut bot = String::new();
    for v in &vals {
        let v = v.clamp(0.0, 100.0);
        let level = ((v / 100.0) * (2 * n) as f64) as usize;
        let level = level.min(2 * n);
        if level <= n {
            top.push(SPARK[0]);
            bot.push(SPARK[level]);
        } else {
            bot.push(SPARK[n]);
            top.push(SPARK[level - n]);
        }
    }
    (top, bot)
}

fn sparkline_down(values: &VecDeque<f64>, width: usize) -> String {
    if values.is_empty() {
        return String::new();
    }
    let vals: Vec<f64> = if values.len() > width {
        values.iter().skip(values.len() - width).copied().collect()
    } else {
        values.iter().copied().collect()
    };
    let mut out = String::new();
    for v in &vals {
        let v = v.clamp(0.0, 100.0);
        let idx = (v / 100.0 * (SPARK_DOWN.len() as f64 - 1.0)) as usize;
        out.push(SPARK_DOWN[idx.min(SPARK_DOWN.len() - 1)]);
    }
    out
}

// ── Bar rendering ──

fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Green => (0, 128, 0),
        Color::Yellow => (255, 255, 0),
        Color::Red => (255, 0, 0),
        Color::Cyan => (0, 255, 255),
        Color::Magenta => (255, 0, 255),
        Color::White => (255, 255, 255),
        Color::LightGreen => (144, 238, 144),
        Color::LightYellow => (255, 255, 224),
        Color::LightRed => (255, 128, 128),
        Color::LightCyan => (224, 255, 255),
        Color::LightMagenta => (255, 128, 255),
        Color::DarkGray => (128, 128, 128),
        _ => (255, 255, 255),
    }
}

fn lerp_color(c1: Color, c2: Color, t: f64) -> Color {
    let (r1, g1, b1) = color_to_rgb(c1);
    let (r2, g2, b2) = color_to_rgb(c2);
    let r = (r1 as f64 + (r2 as f64 - r1 as f64) * t) as u8;
    let g = (g1 as f64 + (g2 as f64 - g1 as f64) * t) as u8;
    let b = (b1 as f64 + (b2 as f64 - b1 as f64) * t) as u8;
    Color::Rgb(r, g, b)
}

fn truecolor_enabled(state: &AppState) -> bool {
    state.color_mode == crate::config::ColorMode::Truecolor
}

fn display_color(c: Color, truecolor: bool) -> Color {
    if truecolor {
        c
    } else {
        basic_color(c)
    }
}

fn basic_color(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => rgb_to_basic_color(r, g, b),
        Color::Indexed(n) => indexed_to_basic_color(n),
        Color::Black | Color::DarkGray => Color::White,
        Color::Gray => Color::White,
        Color::Reset => Color::Reset,
        _ => c,
    }
}

fn indexed_to_basic_color(n: u8) -> Color {
    match n {
        0 | 8 => Color::White,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::White,
        _ => Color::White,
    }
}

fn rgb_to_basic_color(r: u8, g: u8, b: u8) -> Color {
    const CANDIDATES: &[(Color, (i32, i32, i32))] = &[
        (Color::Red, (205, 49, 49)),
        (Color::Green, (13, 188, 121)),
        (Color::Yellow, (229, 229, 16)),
        (Color::Blue, (36, 114, 200)),
        (Color::Magenta, (188, 63, 188)),
        (Color::Cyan, (17, 168, 205)),
        (Color::White, (229, 229, 229)),
        (Color::LightRed, (241, 76, 76)),
        (Color::LightGreen, (35, 209, 139)),
        (Color::LightYellow, (245, 245, 67)),
        (Color::LightBlue, (59, 142, 234)),
        (Color::LightMagenta, (214, 112, 214)),
        (Color::LightCyan, (41, 184, 219)),
    ];

    let r = i32::from(r);
    let g = i32::from(g);
    let b = i32::from(b);
    CANDIDATES
        .iter()
        .min_by_key(|(_, (cr, cg, cb))| {
            let dr = r - cr;
            let dg = g - cg;
            let db = b - cb;
            dr * dr + dg * dg + db * db
        })
        .map(|(color, _)| *color)
        .unwrap_or(Color::White)
}

fn muted_style(steady: bool, truecolor: bool) -> Style {
    if steady || !truecolor {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn muted_bold_style(steady: bool, truecolor: bool) -> Style {
    muted_style(steady, truecolor).add_modifier(Modifier::BOLD)
}

fn empty_bar_span(width: usize, steady: bool, truecolor: bool) -> Span<'static> {
    if steady || !truecolor {
        Span::raw(" ".repeat(width))
    } else {
        Span::styled("░".repeat(width), Style::default().fg(Color::DarkGray))
    }
}

fn bar_spans(
    pct: f64,
    width: usize,
    theme: &Theme,
    steady: bool,
    truecolor: bool,
) -> Vec<Span<'static>> {
    let pct = pct.clamp(0.0, 100.0);
    let filled = (pct / 100.0 * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    let mut spans = Vec::new();
    if steady {
        if filled > 0 {
            spans.push(Span::styled(
                "█".repeat(filled),
                Style::default().fg(display_color(color_for_pct(pct, theme), truecolor)),
            ));
        }
        if empty > 0 {
            spans.push(empty_bar_span(empty, true, truecolor));
        }
        return spans;
    }

    for i in 0..filled {
        let t = i as f64 / (width.max(1) - 1) as f64;
        let c = lerp_color(theme.bar_low, theme.bar_high, t);
        spans.push(Span::styled(
            "█",
            Style::default().fg(display_color(c, truecolor)),
        ));
    }
    if empty > 0 {
        spans.push(empty_bar_span(empty, steady, truecolor));
    }
    spans
}

fn color_for_pct(pct: f64, theme: &Theme) -> Color {
    if pct < 50.0 {
        theme.bar_low
    } else if pct < 80.0 {
        theme.bar_mid
    } else {
        theme.bar_high
    }
}

fn fmt_bytes(b: f64) -> String {
    let mb = b / (1024.0 * 1024.0);
    if mb >= 1000.0 {
        format!("{:.1} GB", mb / 1024.0)
    } else {
        format!("{:.1} MB", mb)
    }
}

fn fmt_speed(b: f64) -> String {
    if b >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} GB/s", b / (1024.0 * 1024.0 * 1024.0))
    } else if b >= 1024.0 * 1024.0 {
        format!("{:.1} MB/s", b / (1024.0 * 1024.0))
    } else if b >= 1024.0 {
        format!("{:.1} KB/s", b / 1024.0)
    } else {
        format!("{:.0} B/s", b)
    }
}

// ── Helpers ──

fn styled_block<'a>(title: &str, color: Color, truecolor: bool) -> Block<'a> {
    let color = display_color(color, truecolor);
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
}

fn with_margin(area: Rect) -> Rect {
    area.inner(Margin {
        vertical: 0,
        horizontal: 1,
    })
}

// ── Main render function ──

pub fn render(f: &mut Frame, state: &AppState) {
    if state.picking_theme {
        render_theme_picker(f, state);
        return;
    }

    let area = f.area();

    // Main vertical layout: GPU | mid(net+cpu+mem) | temps | procs | status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(2),   // GPU
            Constraint::Fill(2),   // mid row
            Constraint::Length(3), // temps
            Constraint::Fill(3),   // procs
            Constraint::Length(1), // status bar
        ])
        .split(area);

    render_gpu(f, with_margin(chunks[0]), state);
    render_mid_row(f, with_margin(chunks[1]), state);
    render_temps(f, with_margin(chunks[2]), state);
    render_procs(f, with_margin(chunks[3]), state);
    render_status_bar(f, with_margin(chunks[4]), state);
}

fn render_gpu(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let truecolor = truecolor_enabled(state);
    let gpus = &state.gpu_infos;

    if gpus.is_empty() {
        let block = styled_block("GPU", theme.gpu, truecolor);
        let text = Paragraph::new(
            "No GPUs detected (install NVIDIA drivers for NVIDIA, or load amdgpu driver for AMD)",
        )
        .style(muted_style(state.steady_render, truecolor))
        .block(block);
        f.render_widget(text, area);
        return;
    }

    let constraints: Vec<Constraint> = gpus
        .iter()
        .map(|_| Constraint::Ratio(1, gpus.len() as u32))
        .collect();
    let gpu_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, g) in gpus.iter().enumerate() {
        let inner_w = gpu_chunks[i].width.saturating_sub(4) as usize;
        let bar_w = inner_w.saturating_sub(14).max(5);
        let spark_w = inner_w.saturating_sub(6).max(10);

        let uc = color_for_pct(g.util, theme);
        let mc = color_for_pct(g.mem_pct, theme);

        let util_bar = bar_spans(g.util, bar_w, theme, state.steady_render, truecolor);
        let mem_bar = bar_spans(g.mem_pct, bar_w, theme, state.steady_render, truecolor);

        let spark_u = if state.steady_render {
            String::new()
        } else {
            state
                .gpu_util_hist
                .get(&g.id)
                .map(|h| sparkline(h, spark_w))
                .unwrap_or_default()
        };
        let spark_m = if state.steady_render {
            String::new()
        } else {
            state
                .gpu_mem_hist
                .get(&g.id)
                .map(|h| sparkline(h, spark_w))
                .unwrap_or_default()
        };

        let mut lines = Vec::new();

        // Util line
        let mut util_line = vec![Span::styled(
            "Util ",
            Style::default().add_modifier(Modifier::BOLD),
        )];
        util_line.extend(util_bar);
        util_line.push(Span::styled(
            format!(" {:5.1}%", g.util),
            Style::default().fg(display_color(uc, truecolor)),
        ));
        lines.push(Line::from(util_line));

        // Util sparkline
        lines.push(Line::from(Span::styled(
            format!("     {}", spark_u),
            Style::default().fg(display_color(uc, truecolor)),
        )));
        lines.push(Line::from(""));

        // Mem line
        let mut mem_line = vec![Span::styled(
            "Mem  ",
            Style::default().add_modifier(Modifier::BOLD),
        )];
        mem_line.extend(mem_bar);
        mem_line.push(Span::styled(
            format!(" {:5.1}%", g.mem_pct),
            Style::default().fg(display_color(mc, truecolor)),
        ));
        lines.push(Line::from(mem_line));

        // Mem info
        lines.push(Line::from(format!(
            "     {:.1}/{:.1} GB",
            g.mem_used_gb, g.mem_total_gb
        )));

        // Mem sparkline
        lines.push(Line::from(Span::styled(
            format!("     {}", spark_m),
            Style::default().fg(display_color(mc, truecolor)),
        )));

        let name_short = g
            .name
            .replace("NVIDIA ", "")
            .replace("AMD ", "")
            .replace("Advanced Micro Devices, Inc. ", "")
            .replace(" Generation", "");

        let block = styled_block(&format!("GPU {}", g.id), theme.gpu, truecolor).title_bottom(
            Line::from(Span::styled(
                format!(" {} ", name_short),
                muted_style(state.steady_render, truecolor),
            )),
        );

        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, gpu_chunks[i]);
    }
}

fn render_mid_row(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    render_net(f, chunks[0], state);
    render_cpu(f, chunks[1], state);
    render_mem(f, chunks[2], state);
}

fn render_cpu(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let truecolor = truecolor_enabled(state);
    let pct = state.cpu_pct;
    let c = color_for_pct(pct, theme);
    let inner_w = area.width.saturating_sub(4) as usize;
    let bar_w = inner_w.saturating_sub(18).max(5);
    let spark_w = inner_w.saturating_sub(10).max(10);

    let cpu_bar = bar_spans(pct, bar_w, theme, state.steady_render, truecolor);
    let (spark_top, spark_bot) = if state.steady_render {
        (String::new(), String::new())
    } else {
        sparkline_double(&state.cpu_hist, spark_w)
    };

    let mut lines = Vec::new();
    let mut overall = vec![Span::styled(
        "Overall  ",
        Style::default().add_modifier(Modifier::BOLD),
    )];
    overall.extend(cpu_bar);
    overall.push(Span::styled(
        format!(" {:5.1}%", pct),
        Style::default().fg(display_color(c, truecolor)),
    ));
    lines.push(Line::from(overall));

    lines.push(Line::from(Span::styled(
        format!(
            "Cores: {}  Total: {:.0}%  Freq: {}",
            state.cpu_cores, state.cpu_total_pct, state.cpu_freq
        ),
        muted_style(state.steady_render, truecolor),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "History",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("         {}", spark_top),
        Style::default().fg(display_color(c, truecolor)),
    )));
    lines.push(Line::from(Span::styled(
        format!("         {}", spark_bot),
        Style::default().fg(display_color(c, truecolor)),
    )));

    let block = styled_block("CPU", theme.cpu, truecolor);

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_net(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let truecolor = truecolor_enabled(state);
    let (up, down) = (state.net_up, state.net_down);
    let mx = state.net_max_speed;
    let up_pct = if mx > 0.0 {
        (up / mx * 100.0).min(100.0)
    } else {
        0.0
    };
    let down_pct = if mx > 0.0 {
        (down / mx * 100.0).min(100.0)
    } else {
        0.0
    };

    let inner_w = area.width.saturating_sub(4) as usize;
    let bar_w = inner_w.saturating_sub(18).max(5);
    let spark_w = inner_w.saturating_sub(6).max(10);

    let up_bar = bar_spans(up_pct, bar_w, theme, state.steady_render, truecolor);
    let down_bar = bar_spans(down_pct, bar_w, theme, state.steady_render, truecolor);

    let (spark_up_str, spark_dn_str) = if state.steady_render {
        (String::new(), String::new())
    } else {
        let up_hist_pct: VecDeque<f64> = state
            .net_up_hist
            .iter()
            .map(|v| (v / mx * 100.0).min(100.0))
            .collect();
        let down_hist_pct: VecDeque<f64> = state
            .net_down_hist
            .iter()
            .map(|v| (v / mx * 100.0).min(100.0))
            .collect();
        (
            sparkline(&up_hist_pct, spark_w),
            sparkline_down(&down_hist_pct, spark_w),
        )
    };

    let mut lines = Vec::new();

    let mut up_line = vec![Span::styled(
        "Up   ",
        Style::default().add_modifier(Modifier::BOLD),
    )];
    up_line.extend(up_bar);
    up_line.push(Span::styled(
        format!(" {:>10}", fmt_speed(up)),
        Style::default().fg(display_color(theme.net_up, truecolor)),
    ));
    lines.push(Line::from(up_line));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("     {}", spark_up_str),
        Style::default().fg(display_color(theme.net_up, truecolor)),
    )));
    lines.push(Line::from(Span::styled(
        format!("     {}", spark_dn_str),
        Style::default().fg(display_color(theme.net_down, truecolor)),
    )));
    lines.push(Line::from(""));

    let mut down_line = vec![Span::styled(
        "Down ",
        Style::default().add_modifier(Modifier::BOLD),
    )];
    down_line.extend(down_bar);
    down_line.push(Span::styled(
        format!(" {:>10}", fmt_speed(down)),
        Style::default().fg(display_color(theme.net_down, truecolor)),
    ));
    lines.push(Line::from(down_line));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("Peak: {}", fmt_speed(mx)),
        muted_style(state.steady_render, truecolor),
    )));

    let block = styled_block("Network", theme.net, truecolor);

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_mem(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let truecolor = truecolor_enabled(state);
    let inner_w = area.width.saturating_sub(4) as usize;
    let bar_w = inner_w.saturating_sub(14).max(5);

    let ram_pct = state.ram_pct;
    let rc = color_for_pct(ram_pct, theme);
    let ram_bar = bar_spans(ram_pct, bar_w, theme, state.steady_render, truecolor);

    let swap_pct = state.swap_pct;
    let swap_bar = bar_spans(swap_pct, bar_w, theme, state.steady_render, truecolor);

    let mut lines = Vec::new();
    let mut ram_line = vec![Span::styled(
        "RAM  ",
        Style::default().add_modifier(Modifier::BOLD),
    )];
    ram_line.extend(ram_bar);
    ram_line.push(Span::styled(
        format!(" {:5.1}%", ram_pct),
        Style::default().fg(display_color(rc, truecolor)),
    ));
    lines.push(Line::from(ram_line));
    lines.push(Line::from(format!(
        "  {} used / {}",
        fmt_bytes(state.ram_used as f64),
        fmt_bytes(state.ram_total as f64)
    )));
    lines.push(Line::from(""));

    let mut swap_line = vec![Span::styled(
        "Swap ",
        Style::default().add_modifier(Modifier::BOLD),
    )];
    swap_line.extend(swap_bar);
    swap_line.push(Span::styled(
        format!(" {:5.1}%", swap_pct),
        muted_style(state.steady_render, truecolor),
    ));
    lines.push(Line::from(swap_line));
    lines.push(Line::from(Span::styled(
        format!(
            "  {} used / {}",
            fmt_bytes(state.swap_used as f64),
            fmt_bytes(state.swap_total as f64)
        ),
        muted_style(state.steady_render, truecolor),
    )));
    lines.push(Line::from(Span::styled(
        format!(
            "  {} avail  Cache: {}",
            fmt_bytes(state.ram_available as f64),
            fmt_bytes(state.ram_cached as f64)
        ),
        muted_style(state.steady_render, truecolor),
    )));

    let block = styled_block("Memory", theme.mem, truecolor);

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_temps(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let truecolor = truecolor_enabled(state);
    let mut cells: Vec<Vec<Span>> = Vec::new();

    if let Some(cpu_t) = state.cpu_temp {
        cells.push(temp_cell(
            "CPU",
            cpu_t,
            state.cpu_temp_max,
            theme,
            state.steady_render,
            truecolor,
        ));
    }
    if let Some(mem_t) = state.mem_temp {
        cells.push(temp_cell(
            "MEM",
            mem_t,
            state.mem_temp_max,
            theme,
            state.steady_render,
            truecolor,
        ));
    }
    for (i, gt) in state.gpu_temps.iter().enumerate() {
        if let Some(t) = gt {
            cells.push(temp_cell(
                &format!("GPU{}", i),
                t.temp,
                t.max,
                theme,
                state.steady_render,
                truecolor,
            ));
        } else {
            cells.push(temp_cell_na(
                &format!("GPU{}", i),
                state.steady_render,
                truecolor,
            ));
        }
    }

    if cells.is_empty() {
        let block = styled_block("Temps", theme.bar_mid, truecolor);
        f.render_widget(
            Paragraph::new("No temperature data")
                .style(muted_style(state.steady_render, truecolor))
                .block(block),
            area,
        );
        return;
    }

    let ncols = cells.len();

    // We render as a single panel
    let block = styled_block("Temps", theme.bar_mid, truecolor);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let constraints: Vec<Constraint> = (0..ncols)
        .map(|_| Constraint::Ratio(1, ncols as u32))
        .collect();
    let temp_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner);

    for (i, cell_spans) in cells.iter().enumerate() {
        if i < temp_cols.len() {
            let line = Line::from(cell_spans.clone());
            f.render_widget(Paragraph::new(vec![line]), temp_cols[i]);
        }
    }
}

fn temp_cell(
    label: &str,
    temp: f64,
    max: f64,
    theme: &Theme,
    steady: bool,
    truecolor: bool,
) -> Vec<Span<'static>> {
    let pct = (temp / max * 100.0).min(100.0);
    let ratio = temp / max;
    let c = if ratio < 0.6 {
        theme.bar_low
    } else if ratio < 0.85 {
        theme.bar_mid
    } else {
        theme.bar_high
    };
    let filled = (pct / 100.0 * 8.0) as usize;
    let c = display_color(c, truecolor);
    vec![
        Span::styled(format!("{} ", label), muted_bold_style(steady, truecolor)),
        Span::styled("█".repeat(filled), Style::default().fg(c)),
        empty_bar_span(8 - filled, steady, truecolor),
        Span::styled(format!(" {:.0}/{:.0}°C", temp, max), Style::default().fg(c)),
    ]
}

fn temp_cell_na(label: &str, steady: bool, truecolor: bool) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!("{} ", label), muted_bold_style(steady, truecolor)),
        Span::styled("N/A", muted_style(steady, truecolor)),
    ]
}

fn render_procs(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_proc_table(f, chunks[0], state, true);
    render_proc_table(f, chunks[1], state, false);
}

fn render_proc_table(f: &mut Frame, area: Rect, state: &AppState, by_mem: bool) {
    let theme = &state.theme;
    let truecolor = truecolor_enabled(state);
    let procs = if by_mem {
        &state.procs_by_mem
    } else {
        &state.procs_by_cpu
    };
    let colour = if by_mem {
        theme.proc_mem
    } else {
        theme.proc_cpu
    };
    let title = if by_mem {
        " Top Processes by Memory "
    } else {
        " Top Processes by CPU "
    };

    let text_style = Style::default();
    let bold = text_style.add_modifier(Modifier::BOLD);
    let header = if by_mem {
        Row::new(vec![
            Cell::from("PID").style(muted_bold_style(state.steady_render, truecolor)),
            Cell::from("Name").style(bold),
            Cell::from("Used").style(bold),
            Cell::from("Shared").style(bold),
            Cell::from("Mem %").style(bold),
        ])
        .style(text_style)
    } else {
        Row::new(vec![
            Cell::from("PID").style(muted_bold_style(state.steady_render, truecolor)),
            Cell::from("Name").style(bold),
            Cell::from("Core %").style(bold),
            Cell::from("CPU %").style(bold),
            Cell::from("Mem %").style(bold),
        ])
        .style(text_style)
    };

    let rows: Vec<Row> = procs
        .iter()
        .map(|p| {
            if by_mem {
                let shared = p.shared;
                let used = if p.rss > shared { p.rss - shared } else { 0 };
                Row::new(vec![
                    Cell::from(format!("{}", p.pid))
                        .style(muted_style(state.steady_render, truecolor)),
                    Cell::from(p.name.clone()),
                    Cell::from(fmt_bytes(used as f64)),
                    Cell::from(fmt_bytes(shared as f64)),
                    Cell::from(format!("{:.1}%", p.memory_percent)),
                ])
                .style(text_style)
            } else {
                let sys_pct = p.cpu_percent / state.num_cpus as f64;
                Row::new(vec![
                    Cell::from(format!("{}", p.pid))
                        .style(muted_style(state.steady_render, truecolor)),
                    Cell::from(p.name.clone()),
                    Cell::from(format!("{:.1}%", p.cpu_percent)),
                    Cell::from(format!("{:.1}%", sys_pct)),
                    Cell::from(format!("{:.1}%", p.memory_percent)),
                ])
                .style(text_style)
            }
        })
        .collect();

    let widths = if by_mem {
        vec![
            Constraint::Length(8),
            Constraint::Min(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(7),
        ]
    } else {
        vec![
            Constraint::Length(8),
            Constraint::Min(10),
            Constraint::Length(8),
            Constraint::Length(7),
            Constraint::Length(7),
        ]
    };

    let table = Table::new(rows, widths)
        .header(header)
        .style(text_style)
        .block(styled_block(title.trim(), colour, truecolor));

    f.render_widget(table, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let truecolor = truecolor_enabled(state);
    let color_text = format!(" {}", state.color_mode.label());
    let left_spans = vec![
        Span::styled(
            " q",
            Style::default()
                .fg(display_color(theme.cpu, truecolor))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("/", muted_style(state.steady_render, truecolor)),
        Span::styled(
            "ESC",
            Style::default()
                .fg(display_color(theme.cpu, truecolor))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Quit  ", muted_style(state.steady_render, truecolor)),
        Span::styled(
            " t",
            Style::default()
                .fg(display_color(theme.gpu, truecolor))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Theme  ", muted_style(state.steady_render, truecolor)),
        Span::styled(
            " c",
            Style::default()
                .fg(display_color(theme.net, truecolor))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Color:", muted_style(state.steady_render, truecolor)),
        Span::styled(
            color_text,
            Style::default()
                .fg(display_color(theme.net, truecolor))
                .add_modifier(Modifier::BOLD),
        ),
    ];

    let power_text = match state.est_power_watts {
        Some(power) => format!("PWR ~{}W  ", power.round() as u64),
        None => "PWR n/a  ".to_string(),
    };
    let oom_text = if let Some(ref oom) = state.oom_str {
        format!("█ OOM Kill: {}", oom)
    } else if state.steady_render {
        "  No OOM kills".to_string()
    } else {
        "░ No OOM kills".to_string()
    };
    let right_len = power_text.len() + oom_text.len();
    let right_width = right_len.min(area.width as usize) as u16;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(right_width)])
        .split(area);

    f.render_widget(Paragraph::new(Line::from(left_spans)), chunks[0]);

    let mut right_spans = Vec::new();
    match state.est_power_watts {
        Some(power) => right_spans.push(Span::styled(
            format!("PWR ~{}W  ", power.round() as u64),
            Style::default()
                .fg(display_color(theme.mem, truecolor))
                .add_modifier(Modifier::BOLD),
        )),
        None => right_spans.push(Span::styled(
            "PWR n/a  ",
            muted_style(state.steady_render, truecolor),
        )),
    }
    if state.oom_str.is_some() {
        right_spans.push(Span::styled(
            oom_text,
            Style::default()
                .fg(display_color(theme.bar_high, truecolor))
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        right_spans.push(Span::styled(
            oom_text,
            muted_style(state.steady_render, truecolor),
        ));
    }

    f.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
        chunks[1],
    );
}

// ── Theme picker ──

fn render_theme_picker(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let names = crate::theme::theme_names();
    let truecolor = truecolor_enabled(state);
    let cols = 3usize;
    let visible_rows = area.height.saturating_sub(8) as usize;
    let total = names.len();
    let cursor = state.theme_cursor;

    let cursor_row = cursor / cols;
    let mut scroll = state.theme_scroll;
    if cursor_row < scroll {
        scroll = cursor_row;
    } else if cursor_row >= scroll + visible_rows {
        scroll = cursor_row.saturating_sub(visible_rows - 1);
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(area);

    // Theme list
    let total_rows = (total + cols - 1) / cols;
    let mut rows = Vec::new();
    for row_idx in scroll..total_rows.min(scroll + visible_rows) {
        let mut cells = Vec::new();
        for col_idx in 0..cols {
            let i = row_idx * cols + col_idx;
            if i < total {
                let name = names[i];
                let th = crate::theme::get_theme(name);
                let gpu = display_color(th.gpu, truecolor);
                let prefix = if i == cursor { " > " } else { "   " };
                let style = if i == cursor {
                    Style::default()
                        .fg(gpu)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else if name == state.theme_name {
                    Style::default().fg(gpu).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(gpu)
                };
                let suffix = if name == state.theme_name && i != cursor {
                    " *"
                } else {
                    ""
                };
                // Build spans with name and swatches
                let line_spans = vec![
                    Span::styled(format!("{}{}{}", prefix, name, suffix), style),
                    Span::raw(" "),
                    Span::styled("  ", Style::default().bg(display_color(th.gpu, truecolor))),
                    Span::raw(" "),
                    Span::styled("  ", Style::default().bg(display_color(th.cpu, truecolor))),
                    Span::raw(" "),
                    Span::styled("  ", Style::default().bg(display_color(th.mem, truecolor))),
                    Span::raw(" "),
                    Span::styled(
                        "  ",
                        Style::default().bg(display_color(th.bar_mid, truecolor)),
                    ),
                ];
                cells.push(Cell::from(Line::from(line_spans)));
            } else {
                cells.push(Cell::from(""));
            }
        }
        rows.push(Row::new(cells));
    }

    let widths = vec![Constraint::Ratio(1, 3); cols];
    let table =
        Table::new(rows, widths).block(styled_block("Select Theme", Color::White, truecolor));
    f.render_widget(table, chunks[0]);

    // Preview
    let preview_name = names.get(cursor).unwrap_or(&"Default");
    let preview = crate::theme::get_theme(preview_name);
    let sample_bar = bar_spans(65.0, 20, &preview, state.steady_render, truecolor);
    let dash = "\u{2501}".repeat(6); // ━ × 6
    let preview_spans = vec![
        Span::styled("Preview: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{}", preview_name),
            Style::default().fg(display_color(preview.gpu, truecolor)),
        ),
        Span::raw("  GPU "),
        Span::styled(
            dash.clone(),
            Style::default().fg(display_color(preview.gpu, truecolor)),
        ),
        Span::raw("  Net "),
        Span::styled(
            dash.clone(),
            Style::default().fg(display_color(preview.net, truecolor)),
        ),
        Span::raw("  CPU "),
        Span::styled(
            dash.clone(),
            Style::default().fg(display_color(preview.cpu, truecolor)),
        ),
        Span::raw("  Mem "),
        Span::styled(
            dash,
            Style::default().fg(display_color(preview.mem, truecolor)),
        ),
    ];
    let preview_line1 = Line::from(preview_spans);
    let mut bar_line_spans = vec![Span::raw("  Bar: ")];
    bar_line_spans.extend(sample_bar);
    let preview_line2 = Line::from(bar_line_spans);

    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(muted_style(state.steady_render, truecolor));
    f.render_widget(
        Paragraph::new(vec![Line::from(""), preview_line1, preview_line2]).block(preview_block),
        chunks[1],
    );

    // Hints
    let hint = Line::from(vec![
        Span::styled(
            " UP/DOWN/LEFT/RIGHT",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Navigate  "),
        Span::styled("ENTER", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Select  "),
        Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel"),
    ]);
    f.render_widget(Paragraph::new(hint), chunks[2]);
}
