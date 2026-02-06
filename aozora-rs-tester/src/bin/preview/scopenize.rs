use std::io;
use std::path::PathBuf;

use ansi_to_tui::IntoText;
use aozora_rs_core::prelude::{scopenize, tokenize};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::Screen;
use crate::app_context::AppContext;
use crate::theme::Theme;

/// A displayable scope with owned data
struct ScopeDisplay {
    deco_name: String,
    span: std::ops::Range<usize>,
}

struct ScopenizeApp {
    original_text: String,
    scopes: Vec<ScopeDisplay>,
    selected: usize,
    scroll_offset: usize,
    text_scroll: usize,
    error_message: Option<String>,
}

impl ScopenizeApp {
    fn new() -> Self {
        Self {
            original_text: String::new(),
            scopes: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            text_scroll: 0,
            error_message: None,
        }
    }

    fn load_and_process(&mut self, context: &AppContext) {
        self.scopes.clear();
        self.selected = 0;
        self.scroll_offset = 0;
        self.text_scroll = 0;
        self.error_message = None;

        match context.load_current_file_content() {
            Ok(text) => {
                self.original_text = text;

                // Tokenize
                let tokens = match tokenize(&self.original_text) {
                    Ok((_, t)) => t,
                    Err(e) => {
                        self.error_message = Some(format!("トークン化エラー: {}", e));
                        return;
                    }
                };

                // Scopenize
                let ((scopes, _flat), _) = scopenize(tokens, &self.original_text).into_tuple();
                self.scopes = scopes
                    .0
                    .into_values()
                    .flatten()
                    .map(|s| ScopeDisplay {
                        deco_name: s.deco.to_string(),
                        span: s.span,
                    })
                    .collect();
                // Sort by start position
                self.scopes.sort_by_key(|s| s.span.start);
            }
            Err(e) => {
                self.error_message = Some(format!("読み込みエラー: {}", e));
            }
        }
    }

    fn next_file(&mut self, context: &mut AppContext) {
        context.next_file();
        self.load_and_process(context);
    }

    fn prev_file(&mut self, context: &mut AppContext) {
        context.prev_file();
        self.load_and_process(context);
    }

    fn next_scope(&mut self) {
        if self.selected < self.scopes.len().saturating_sub(1) {
            self.selected += 1;
            // Always keep selected item at the 2nd row (index 1) relative to scroll window,
            // except when at the very top.
            self.scroll_offset = self.selected.saturating_sub(1);
        }
    }

    fn prev_scope(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.scroll_offset = self.selected.saturating_sub(1);
        }
    }
}

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    context: &mut AppContext,
) -> io::Result<Screen> {
    if context.file_paths.is_empty() {
        return Ok(Screen::Home);
    }

    let mut app = ScopenizeApp::new();
    app.load_and_process(context);

    loop {
        terminal.draw(|f| ui(f, &app, context))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Ok(Screen::Home),
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(Screen::Exit);
                    }
                    KeyCode::Left => app.prev_file(context),
                    KeyCode::Right => app.next_file(context),
                    KeyCode::Up => app.prev_scope(),
                    KeyCode::Down => {
                        app.next_scope();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &ScopenizeApp, context: &AppContext) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(1),     // Text with scope layers (Main)
        Constraint::Length(10), // Scope list (Sub)
    ])
    .split(f.area());

    // Header
    let file_name = context
        .current_file()
        .and_then(|p| {
            PathBuf::from(p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "不明".to_string());

    let header_text = vec![
        Line::from(vec![
            Span::styled(
                "Scopenize プレビュー - ",
                Style::default().fg(Theme::HEADER_FG_TITLE),
            ),
            Span::styled(&file_name, Style::default().fg(Theme::HEADER_FG_FILE)),
            Span::styled(
                format!(
                    " ({}/{})",
                    context.current_index + 1,
                    context.file_paths.len()
                ),
                Style::default().fg(Theme::HEADER_FG_INFO),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "←/→: ファイル切替  ",
                Style::default().fg(Theme::STATUS_BAR_FG),
            ),
            Span::styled(
                "↑/↓: スコープ選択  ",
                Style::default().fg(Theme::STATUS_BAR_FG),
            ),
            Span::styled("Esc: 戻る", Style::default().fg(Theme::STATUS_BAR_FG)),
        ]),
    ];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Text with scope visualization
    if let Some(ref err) = app.error_message {
        let text = err.into_text().unwrap();
        let error =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("エラー"));
        f.render_widget(error, chunks[1]);
    } else {
        render_text_with_scopes(f, app, chunks[1]);
    }

    // Scope list
    render_scope_list(f, app, chunks[2]);
}

fn render_text_with_scopes(f: &mut Frame, app: &ScopenizeApp, area: Rect) {
    use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

    let inner_width = area.width.saturating_sub(7) as usize; // Reserve space for line numbers
    let inner_height = area.height.saturating_sub(2) as usize;

    if inner_height == 0 || inner_width == 0 {
        return;
    }

    // Determine target line based on selected scope
    let target_line_idx = if let Some(scope) = app.scopes.get(app.selected) {
        byte_offset_to_line_idx(&app.original_text, scope.span.start)
    } else {
        0
    };

    // Auto-scroll logic (Anchor to top)
    let start_line_idx = target_line_idx.saturating_sub(2);

    let lines: Vec<&str> = app.original_text.lines().collect();
    let mut display_lines: Vec<Line> = Vec::new();

    // Calculate line byte offsets
    let mut line_offsets = Vec::new();
    let mut running_offset = 0;
    for line in &lines {
        line_offsets.push(running_offset);
        running_offset += line.len();
        while running_offset < app.original_text.len() {
            let c = app.original_text[running_offset..].chars().next().unwrap();
            if c == '\r' || c == '\n' {
                running_offset += c.len_utf8();
            } else {
                break;
            }
        }
    }

    let colors = [
        Color::Yellow,
        Color::Magenta,
        Color::Cyan,
        Color::Green,
        Color::Red,
        Color::Blue,
    ];

    for (i, line) in lines.iter().enumerate().skip(start_line_idx) {
        if display_lines.len() >= inner_height {
            break;
        }

        let line_start_byte = line_offsets[i];
        let line_end_byte = line_start_byte + line.len();

        // Find relevant scopes (byte ranges relative to line start)
        let relevant_scopes: Vec<(usize, std::ops::Range<usize>)> = app
            .scopes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.span.start < line_end_byte && s.span.end > line_start_byte)
            .map(|(idx, s)| {
                let s_byte = s.span.start.max(line_start_byte) - line_start_byte;
                let e_byte = s.span.end.min(line_end_byte) - line_start_byte;
                (idx, s_byte..e_byte)
            })
            .collect();

        // Wrap logic: accumulate chars until visual width limit
        let mut byte_idx_in_line = 0;

        loop {
            if display_lines.len() >= inner_height {
                break;
            }
            if byte_idx_in_line >= line.len() && !line.is_empty() {
                break;
            }
            if line.is_empty() {
                // Handle empty line
                let line_num = format!("{:4} ", i + 1);
                display_lines.push(Line::from(vec![
                    Span::styled(line_num, Style::default().fg(Theme::TEXT_LINENUM_FG)),
                    Span::raw(""),
                ]));
                break;
            }

            // Determine chunk end
            let chunk_start_byte = byte_idx_in_line;
            let mut current_width = 0;
            let mut chunk_end_byte = byte_idx_in_line;

            // Greedily consume chars
            let mut chunk_chars = String::new();
            while chunk_end_byte < line.len() {
                let c = line[chunk_end_byte..].chars().next().unwrap();
                let w = c.width().unwrap_or(0);
                if current_width + w > inner_width && current_width > 0 {
                    // Must wrap here
                    break;
                }
                current_width += w;
                chunk_end_byte += c.len_utf8();
                chunk_chars.push(c);
            }

            // Add text line
            let line_num_str = if chunk_start_byte == 0 {
                format!("{:4} ", i + 1)
            } else {
                "     ".to_string()
            };
            display_lines.push(Line::from(vec![
                Span::styled(line_num_str, Style::default().fg(Theme::TEXT_LINENUM_FG)),
                Span::raw(chunk_chars.clone()),
            ]));

            // Scope markers for this chunk
            let chunk_scopes: Vec<(usize, usize, usize)> = relevant_scopes
                .iter()
                .filter(|(_, range)| range.start < chunk_end_byte && range.end > chunk_start_byte)
                .map(|(idx, range)| {
                    // Clip to chunk
                    let s = range.start.max(chunk_start_byte);
                    let e = range.end.min(chunk_end_byte);

                    // We need display WIDTH offsets relative to chunk start
                    let prefix = &line[chunk_start_byte..s];
                    let body = &line[s..e];

                    let indent = prefix.width();
                    let len = body.width();

                    (*idx, indent, len)
                })
                .collect();

            if !chunk_scopes.is_empty() {
                // Pack
                let mut rows: Vec<Vec<(usize, usize, usize)>> = Vec::new(); // indent, len, scope_idx
                for (scope_idx, indent, len) in chunk_scopes {
                    let mut placed = false;
                    for row in &mut rows {
                        let last_end_width = row.last().map(|(i, l, _)| i + l).unwrap_or(0);
                        if indent >= last_end_width + 1 {
                            // space
                            row.push((indent, len, scope_idx));
                            placed = true;
                            break;
                        }
                    }
                    if !placed {
                        rows.push(vec![(indent, len, scope_idx)]);
                    }
                }

                // Render marker rows
                for (row_idx, row) in rows.iter().enumerate() {
                    if display_lines.len() >= inner_height {
                        break;
                    }

                    let mut spans = vec![Span::raw("     ")];
                    let mut last_width_pos = 0;
                    let color = colors[row_idx % colors.len()];

                    for (indent, len, idx) in row {
                        if *indent > last_width_pos {
                            spans.push(Span::raw(" ".repeat(*indent - last_width_pos)));
                        }

                        let style = if *idx == app.selected {
                            Style::default().fg(color).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(color)
                        };

                        spans.push(Span::styled("~".repeat(*len), style));
                        last_width_pos = *indent + *len;
                    }
                    display_lines.push(Line::from(spans));
                }
            }

            // Prepare next chunk
            byte_idx_in_line = chunk_end_byte;
        }
    }

    let paragraph = Paragraph::new(display_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Text (Line {}) ", target_line_idx + 1)),
    );
    f.render_widget(paragraph, area);
}

fn byte_offset_to_line_idx(text: &str, byte_offset: usize) -> usize {
    text.char_indices()
        .take_while(|(idx, _)| *idx < byte_offset)
        .filter(|(_, c)| *c == '\n')
        .count()
}

fn render_scope_list(f: &mut Frame, app: &ScopenizeApp, area: Rect) {
    let visible_rows = area.height.saturating_sub(2) as usize;

    let items: Vec<ListItem> = app
        .scopes
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(visible_rows)
        .map(|(i, s)| {
            let style = if i == app.selected {
                Theme::selected_style()
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{:3}] ", i),
                    Style::default().fg(Theme::HEADER_FG_INFO),
                ),
                Span::styled(format!("{:<20}", &s.deco_name), style),
                Span::styled(
                    format!(" {:5}..{:5}", s.span.start, s.span.end),
                    Style::default().fg(Theme::HEADER_FG_TITLE), // Reusing Cyan
                ),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected.saturating_sub(app.scroll_offset)));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Scopes ({} 件) ", app.scopes.len())),
        )
        .highlight_style(Theme::highlight_block_style());
    f.render_stateful_widget(list, area, &mut state);
}
