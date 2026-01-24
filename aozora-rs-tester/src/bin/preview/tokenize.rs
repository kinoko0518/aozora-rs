use std::io;
use std::path::PathBuf;

use aozora_rs::prelude::{AozoraTokenKind, Input, tokenize};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use winnow::{LocatingSlice, Parser};

use super::Screen;
use crate::app_context::AppContext;

struct TokenDisplay {
    kind_name: String,
    span: std::ops::Range<usize>,
    content: String,
}

struct TokenizeApp {
    original_text: String,
    tokens: Vec<TokenDisplay>,
    selected: usize,
    scroll_offset: usize,
    error_message: Option<String>,
}

impl TokenizeApp {
    fn new() -> Self {
        Self {
            original_text: String::new(),
            tokens: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            error_message: None,
        }
    }

    fn load_current_file(&mut self, context: &AppContext) {
        self.selected = 0;
        self.scroll_offset = 0;
        self.error_message = None;
        self.tokens.clear();

        match context.load_current_file_content() {
            Ok(text) => {
                self.original_text = text;
                // Tokenize
                let mut input: Input = LocatingSlice::new(&self.original_text);
                match tokenize.parse_next(&mut input) {
                    Ok(toks) => {
                        self.tokens = toks
                            .into_iter()
                            .map(|t| {
                                let content: String = self
                                    .original_text
                                    .get(t.span.clone())
                                    .unwrap_or("")
                                    .chars()
                                    .take(50)
                                    .collect();
                                TokenDisplay {
                                    kind_name: format_token_kind(&t.kind),
                                    span: t.span,
                                    content,
                                }
                            })
                            .collect();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("トークン化エラー: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("読み込みエラー: {}", e));
            }
        }
    }

    fn next_file(&mut self, context: &mut AppContext) {
        context.next_file();
        self.load_current_file(context);
    }

    fn prev_file(&mut self, context: &mut AppContext) {
        context.prev_file();
        self.load_current_file(context);
    }

    fn next_token(&mut self, visible_rows: usize) {
        if self.selected < self.tokens.len().saturating_sub(1) {
            self.selected += 1;
            if self.selected >= self.scroll_offset + visible_rows {
                self.scroll_offset = self.selected - visible_rows + 1;
            }
        }
    }

    fn prev_token(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
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

    let mut app = TokenizeApp::new();
    app.load_current_file(context);

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
                    KeyCode::Up => app.prev_token(),
                    KeyCode::Down => {
                        let visible = terminal.size()?.height.saturating_sub(8) as usize;
                        app.next_token(visible);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn format_token_kind(kind: &AozoraTokenKind) -> String {
    match kind {
        AozoraTokenKind::Text(_) => "Text".to_string(),
        AozoraTokenKind::Ruby(_) => "Ruby".to_string(),
        AozoraTokenKind::RubyDelimiter => "RubyDelimiter".to_string(),
        AozoraTokenKind::Command(_) => "Command".to_string(),
        AozoraTokenKind::Odoriji(_) => "Odoriji".to_string(),
        AozoraTokenKind::Br => "Br".to_string(),
    }
}

fn ui(f: &mut Frame, app: &TokenizeApp, context: &AppContext) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(6),
    ])
    .split(f.area());

    // Header with file name
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
            Span::styled("Tokenize プレビュー - ", Style::default().fg(Color::Cyan)),
            Span::styled(&file_name, Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    " ({}/{})",
                    context.current_index + 1,
                    context.file_paths.len()
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("←/→: ファイル切替  ", Style::default().fg(Color::Gray)),
            Span::styled("↑/↓: トークン選択  ", Style::default().fg(Color::Gray)),
            Span::styled("Esc: 戻る", Style::default().fg(Color::Gray)),
        ]),
    ];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Token list
    if let Some(ref err) = app.error_message {
        let error = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("エラー"));
        f.render_widget(error, chunks[1]);
    } else {
        render_token_list(f, app, chunks[1]);
    }

    // Detail panel
    render_detail(f, app, chunks[2]);
}

fn render_token_list(f: &mut Frame, app: &TokenizeApp, area: Rect) {
    let visible_rows = area.height.saturating_sub(2) as usize;

    let items: Vec<ListItem> = app
        .tokens
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(visible_rows)
        .map(|(i, t)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{:4}] ", i), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<14}", t.kind_name), style),
                Span::styled(
                    format!(" ({:5}..{:5})", t.span.start, t.span.end),
                    Style::default().fg(Color::Cyan),
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
                .title(format!(" Tokens ({} 件) ", app.tokens.len())),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_detail(f: &mut Frame, app: &TokenizeApp, area: Rect) {
    let detail = if let Some(t) = app.tokens.get(app.selected) {
        vec![
            Line::from(vec![
                Span::styled("種類: ", Style::default().fg(Color::Gray)),
                Span::styled(&t.kind_name, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("範囲: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}..{}", t.span.start, t.span.end),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("内容: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    t.content.replace('\n', "↵").replace('\r', ""),
                    Style::default().fg(Color::White),
                ),
            ]),
        ]
    } else {
        vec![Line::from("トークンを選択してください")]
    };

    let paragraph =
        Paragraph::new(detail).block(Block::default().borders(Borders::ALL).title(" 詳細 "));
    f.render_widget(paragraph, area);
}
