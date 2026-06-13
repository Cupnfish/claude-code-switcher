//! ratatui rendering for the apply TUI.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::credentials::{ApiKeySource, mask_api_key};
use crate::tui::app::{App, Mode};
use crate::tui::input::TextInput;

const CURSOR: &str = "❯ ";
const NONE_CURSOR: &str = "  ";

fn cursor_prefix(is_cursor: bool) -> &'static str {
    if is_cursor {
        CURSOR
    } else {
        NONE_CURSOR
    }
}

/// A line that may be a selectable row (carrying its row index) or decoration.
struct RowLine {
    #[allow(dead_code)]
    row: Option<usize>,
    line: Line<'static>,
}

fn header(text: &str) -> RowLine {
    RowLine {
        row: None,
        line: Line::styled(
            format!(" {text}"),
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM),
        ),
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Layout: title (3) | body (flex) | help (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    render_title(frame, app, chunks[0]);
    render_body(frame, app, chunks[1]);
    render_help(frame, chunks[2]);

    render_popup(frame, app);
}

fn render_title(frame: &mut Frame, app: &App, area: Rect) {
    let title = format!(" Apply  {}  ·  {} ", app.target, app.display_name);
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let right = format!("switching from: {} ", app.current_label);

    let line = Line::from(vec![
        Span::styled(title.clone(), Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(right, Style::default().fg(Color::DarkGray)),
    ]);
    let p = Paragraph::new(line).block(block);
    frame.render_widget(p, area);
}

fn render_body(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " ↑/↓ move · ←/→ change · Enter select/apply · a apply ",
            Style::default().fg(Color::DarkGray),
        ));

    let mut rows: Vec<RowLine> = Vec::new();

    // Key section
    rows.push(header("Key"));
    let mut row_idx = 0usize;
    for (i, src) in app.sources().iter().enumerate() {
        let is_sel = app.selected_key() == Some(i);
        let is_cur = app.cursor() == row_idx;
        let mark = if is_sel { "●" } else { "○" };
        let (icon, name, detail) = match src {
            ApiKeySource::EnvVar {
                env_var_name,
                api_key,
            } => (
                "🌐",
                env_var_name.clone(),
                format!("{} (env)", mask_api_key(api_key)),
            ),
            ApiKeySource::Saved { credential } => {
                let mut d = mask_api_key(credential.api_key());
                if credential.last_used_at().is_some() {
                    d.push_str("  · last used");
                }
                ("🔑", credential.name().to_string(), d)
            }
        };
        let _ = icon;
        let line = Line::from(vec![
            Span::raw(cursor_prefix(is_cur).to_string()),
            Span::styled(
                format!("{mark} "),
                Style::default().fg(if is_sel {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled(
                format!("{name:<16}"),
                if is_cur {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(format!(" {detail}"), Style::default().fg(Color::DarkGray)),
        ]);
        rows.push(RowLine {
            row: Some(row_idx),
            line,
        });
        row_idx += 1;
    }
    // new-key row
    {
        let is_cur = app.cursor() == row_idx;
        let line = Line::from(vec![
            Span::raw(cursor_prefix(is_cur).to_string()),
            Span::styled("➕ ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "enter a new key...".to_string(),
                if is_cur {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]);
        rows.push(RowLine {
            row: Some(row_idx),
            line,
        });
        row_idx += 1;
    }

    // Options section
    rows.push(header("Options   (←/→ to change)"));
    let cur = app.cursor();
    rows.push(option_row("Effort   ", app.effort(), cur == row_idx));
    row_idx += 1;
    rows.push(option_row(
        "Scope    ",
        &app.scope().to_string(),
        cur == row_idx,
    ));
    row_idx += 1;
    rows.push(option_row(
        "Co-author",
        if app.co_author_enabled() { "on" } else { "off" },
        cur == row_idx,
    ));
    row_idx += 1;
    if app.has_variant_row()
        && let Some(label) = app.variant_label() {
            rows.push(option_row("Variant  ", label, cur == row_idx));
            row_idx += 1;
        }

    // Preview section
    rows.push(header("Preview"));
    let (model, base) = app.preview_model_and_base();
    let preview_lines = vec![
        Line::from(vec![
            Span::styled(" model : ", Style::default().fg(Color::DarkGray)),
            Span::raw(model),
            Span::styled("   base : ", Style::default().fg(Color::DarkGray)),
            Span::raw(base),
        ]),
        Line::from(vec![
            Span::styled(" key   : ", Style::default().fg(Color::DarkGray)),
            Span::raw(app.masked_selected_key()),
            Span::styled(
                format!("   effort : {}", app.effort()),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    for pl in preview_lines {
        rows.push(RowLine { row: None, line: pl });
    }

    // Apply row
    let is_cur = app.cursor() == row_idx;
    let apply_line = Line::from(vec![
        Span::raw(cursor_prefix(is_cur).to_string()),
        Span::styled(
            "► Apply".to_string(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "   (a · Enter)".to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    rows.push(RowLine {
        row: Some(row_idx),
        line: apply_line,
    });

    // Render all rows; the `❯` prefix already marks the cursor line.
    let lines: Vec<Line> = rows.iter().map(|r| r.line.clone()).collect();
    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn option_row(label: &str, value: &str, is_cursor: bool) -> RowLine {
    let line = Line::from(vec![
        Span::raw(cursor_prefix(is_cursor).to_string()),
        Span::styled(format!(" {label} "), Style::default().fg(Color::Gray)),
        Span::styled(
            format!("[ {value} ]"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    RowLine { row: None, line }
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help = " ↑/↓ move · ←/→ change option · Enter select/new-key/apply · a apply · n new · d delete · r rename · ? help · Esc quit ";
    let p = Paragraph::new(Line::from(vec![Span::styled(
        help,
        Style::default().fg(Color::DarkGray),
    )]))
    .alignment(Alignment::Center);
    frame.render_widget(p, area);
}

fn render_popup(frame: &mut Frame, app: &App) {
    let mode = app.mode_ref();
    let (title, lines, height): (String, Vec<Line>, u16) = match mode {
        Mode::InputNewKey(input) => input_popup("Create new API key", "Paste your API key:", input),
        Mode::InputRename { input, .. } => input_popup("Rename key", "New name:", input),
        Mode::ConfirmDelete { .. } => (
            "Confirm delete".to_string(),
            vec![
                Line::from(Span::raw("")),
                Line::from(vec![Span::raw("Delete this key? This cannot be undone.")]),
                Line::from(Span::raw("")),
                Line::from(vec![
                    Span::styled(
                        " Enter",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" confirm   "),
                    Span::styled("Esc", Style::default().fg(Color::Yellow)),
                    Span::raw(" cancel"),
                ]),
            ],
            6,
        ),
        Mode::Help => (
            "Help".to_string(),
            help_lines(),
            11,
        ),
        Mode::Message(msg) => (
            "Notice".to_string(),
            vec![
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled(
                    format!(" {msg} "),
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled(
                    "Press any key to dismiss",
                    Style::default().fg(Color::DarkGray),
                )]),
            ],
            6,
        ),
        Mode::Normal => return,
    };

    let area = popup_area(64, height, frame.area());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default());
    frame.render_widget(Clear, area);
    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}

fn input_popup(title: &str, prompt: &str, input: &TextInput) -> (String, Vec<Line<'static>>, u16) {
    let val = input.value();
    let cbyte = input.cursor_byte();
    let before = val[..cbyte.min(val.len())].to_string();
    let (cur_ch, after) = if cbyte < val.len() {
        let mut it = val[cbyte..].chars();
        let ch = it.next().map(|c| c.to_string()).unwrap_or_default();
        (ch, it.collect::<String>())
    } else {
        (" ".to_string(), String::new())
    };
    let lines = vec![
        Line::from(Span::raw("")),
        Line::from(vec![Span::styled(
            prompt.to_string(),
            Style::default().fg(Color::Gray),
        )]),
        Line::from(vec![
            Span::raw(" "),
            Span::raw(before),
            Span::styled(
                cur_ch,
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(after),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled(
                " Enter",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" confirm   "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ]),
    ];
    (title.to_string(), lines, 7)
}

fn help_lines() -> Vec<Line<'static>> {
    let b = |k: &str, d: &str| {
        Line::from(vec![
            Span::styled(format!("  {k:<8}"), Style::default().fg(Color::Cyan)),
            Span::raw(d.to_string()),
        ])
    };
    vec![
        Line::from(Span::raw("")),
        b("↑/↓", "move cursor (key list → options → Apply)"),
        b("←/→", "change the focused option's value"),
        b("Enter", "on a key: select & apply · ➕: new · Apply: apply"),
        b("a", "apply now (selected key + current options)"),
        b("n/d/r", "new / delete / rename key"),
        b("Esc/q", "quit without applying"),
        Line::from(Span::raw("")),
        Line::from(vec![Span::styled(
            " Choices are remembered for next time.",
            Style::default().fg(Color::DarkGray),
        )]),
    ]
}

/// A rect of the given (clamped) size, centered within `area`.
fn popup_area(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}
