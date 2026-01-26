use std::io;
use std::path::PathBuf;

use ansi_to_tui::IntoText;
use aozora_rs::prelude::{Input, Retokenized, scopenize, tokenize};
use aozora_rs::retokenizer::processor::retokenize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use miette::GraphicalReportHandler;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use winnow::{LocatingSlice, Parser};

use super::Screen;
use crate::app_context::AppContext;
use crate::theme::Theme;

struct RetokenizedApp {
    output_text: String,
    scroll_y: u16,
    error_message: Option<String>,
}

impl RetokenizedApp {
    fn new() -> Self {
        Self {
            output_text: String::new(),
            scroll_y: 0,
            error_message: None,
        }
    }

    fn load_and_process(&mut self, context: &AppContext) {
        self.output_text.clear();
        self.scroll_y = 0;
        self.error_message = None;

        match context.load_current_file_content() {
            Ok(text) => {
                // Tokenize
                let mut input: Input = LocatingSlice::new(&text);
                let tokens = match tokenize.parse_next(&mut input) {
                    Ok(t) => t,
                    Err(e) => {
                        self.error_message = Some(format!("トークン化エラー: {}", e));
                        return;
                    }
                };

                // Scopenize
                let (flat, scopes) = match scopenize(tokens, &text) {
                    Ok(res) => res,
                    Err(e) => {
                        let mut buf = String::new();
                        GraphicalReportHandler::new()
                            .render_report(&mut buf, e.as_ref())
                            .unwrap();
                        self.error_message = Some(buf);
                        return;
                    }
                };

                // Retokenize
                match retokenize(flat, scopes) {
                    Ok(retokenized) => {
                        self.format_retokenized(retokenized);
                    }
                    Err(e) => {
                        let mut buf = String::new();
                        GraphicalReportHandler::new()
                            .render_report(&mut buf, e.as_ref())
                            .unwrap();
                        self.error_message = Some(buf);
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("読み込みエラー: {}", e));
            }
        }
    }

    fn format_retokenized(&mut self, tokens: Vec<Retokenized>) {
        let mut buffer = String::new();

        for token in tokens {
            match token {
                Retokenized::Text(t) => buffer.push_str(&t),
                Retokenized::DecoBegin(d) => buffer.push_str(&format!("<{}>", d)),
                Retokenized::DecoEnd(d) => buffer.push_str(&format!("</{}>", d)),
                Retokenized::Break(b) => buffer.push_str(&format!("<{}>", b)),
                Retokenized::Figure(f) => buffer.push_str(&format!("<{}>", f)),
                Retokenized::Odoriji(o) => buffer.push_str(&format!("<{}>", o)),
            }
        }

        self.output_text = buffer;
    }

    fn next_file(&mut self, context: &mut AppContext) {
        context.next_file();
        self.load_and_process(context);
    }

    fn prev_file(&mut self, context: &mut AppContext) {
        context.prev_file();
        self.load_and_process(context);
    }

    fn scroll_up(&mut self) {
        if self.scroll_y > 0 {
            self.scroll_y -= 1;
        }
    }

    fn scroll_down(&mut self) {
        self.scroll_y += 1;
    }
}

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    context: &mut AppContext,
) -> io::Result<Screen> {
    if context.file_paths.is_empty() {
        return Ok(Screen::Home);
    }

    let mut app = RetokenizedApp::new();
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
                    KeyCode::Up => app.scroll_up(),
                    KeyCode::Down => app.scroll_down(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &RetokenizedApp, context: &AppContext) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(1),    // Content
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
                "Retokenize プレビュー - ",
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
                "↑/↓: スクロール  ",
                Style::default().fg(Theme::STATUS_BAR_FG),
            ),
            Span::styled("Esc: 戻る", Style::default().fg(Theme::STATUS_BAR_FG)),
        ]),
    ];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Content
    if let Some(ref err) = app.error_message {
        let text = err.into_text().unwrap();
        let error =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("エラー"));
        f.render_widget(error, chunks[1]);
    } else {
        render_content(f, app, chunks[1]);
    }
}

fn render_content(f: &mut Frame, app: &RetokenizedApp, area: Rect) {
    // 簡易的な行分割（折り返し）処理
    let width = area.width.saturating_sub(2) as usize; // Border分
    if width == 0 {
        return;
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for c in app.output_text.chars() {
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if c == '\n' {
            lines.push(Line::from(current_line));
            current_line = String::new();
            current_width = 0;
            continue;
        }

        if current_width + w > width {
            lines.push(Line::from(current_line));
            current_line = String::new();
            current_width = 0;
        }

        current_line.push(c);
        current_width += w;
    }
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Retokenized Output "),
        )
        .scroll((app.scroll_y, 0));

    f.render_widget(paragraph, area);
}
