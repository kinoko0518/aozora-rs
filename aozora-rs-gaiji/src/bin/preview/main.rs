use std::collections::HashMap;
use std::fs;
use std::io;

use rkyv::Deserialize;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

type GaijiMap = HashMap<String, char>;

struct App {
    items: Vec<(String, char)>,
    filtered_indices: Vec<usize>,
    offset: usize,
    search_query: String,
}

impl App {
    fn new(items: Vec<(String, char)>) -> Self {
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
        Self {
            items,
            filtered_indices,
            offset: 0,
            search_query: String::new(),
        }
    }

    fn is_search_mode(&self) -> bool {
        !self.search_query.is_empty()
    }

    fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            self.filtered_indices = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, (tag, c))| {
                    tag.contains(&self.search_query) || c.to_string().contains(&self.search_query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.offset = 0;
    }

    fn visible_count(&self) -> usize {
        if self.is_search_mode() {
            self.filtered_indices.len()
        } else {
            self.items.len()
        }
    }

    fn scroll_up(&mut self) {
        if self.offset > 0 {
            self.offset -= 1;
        }
    }

    fn scroll_down(&mut self, visible_rows: usize) {
        let max_offset = self.visible_count().saturating_sub(visible_rows);
        if self.offset < max_offset {
            self.offset += 1;
        }
    }

    fn add_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filter();
    }

    fn delete_char(&mut self) {
        self.search_query.pop();
        self.update_filter();
    }
}

fn main() -> io::Result<()> {
    // Load gaiji map
    let bytes = fs::read("./gaiji_to_char.map")?;
    let archived = unsafe { rkyv::archived_root::<GaijiMap>(&bytes) };
    let map: GaijiMap = archived.deserialize(&mut rkyv::Infallible).unwrap();

    // Convert to sorted Vec
    let mut items: Vec<(String, char)> = map.into_iter().collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let mut app = App::new(items);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Up => {
                        app.scroll_up();
                    }
                    KeyCode::Down => {
                        let visible_rows = terminal.size()?.height.saturating_sub(6) as usize;
                        app.scroll_down(visible_rows);
                    }
                    KeyCode::Char(c) => {
                        app.add_char(c);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(1),    // Main area
        Constraint::Length(3), // Search bar
    ])
    .split(f.area());

    // Header
    let header_text = vec![
        Line::from(vec![Span::styled(
            "青空文庫書式外字抽出器　プレビュー",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("↑/↓: スクロール  ", Style::default().fg(Color::Gray)),
            Span::styled("Ctrl+q: 終了", Style::default().fg(Color::Gray)),
        ]),
    ];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Main area
    render_main_area(f, app, chunks[1]);

    // Search bar
    let search_text = format!("検索: {}", app.search_query);
    let search_bar = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.is_search_mode() {
                    Color::Yellow
                } else {
                    Color::White
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(search_bar, chunks[2]);

    // Cursor position in search bar
    let cursor_x =
        chunks[2].x + 1 + "検索: ".chars().count() as u16 + app.search_query.chars().count() as u16;
    let cursor_y = chunks[2].y + 1;
    f.set_cursor_position((cursor_x, cursor_y));
}

fn render_main_area(f: &mut Frame, app: &App, area: Rect) {
    let visible_rows = area.height.saturating_sub(2) as usize;

    if app.is_search_mode() && app.filtered_indices.is_empty() {
        // No matches
        let no_match = Paragraph::new("マッチしませんでした")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(if app.is_search_mode() {
                        "検索結果"
                    } else {
                        "先頭からプレビュー"
                    }),
            );
        f.render_widget(no_match, area);
    } else {
        // Build table rows - O(N) where N is visible_rows
        let rows: Vec<Row> = if app.is_search_mode() {
            app.filtered_indices
                .iter()
                .skip(app.offset)
                .take(visible_rows)
                .map(|&idx| {
                    let (tag, c) = &app.items[idx];
                    Row::new(vec![
                        Cell::from(c.to_string()).style(Style::default().fg(Color::Green)),
                        Cell::from(tag.as_str()),
                    ])
                })
                .collect()
        } else {
            app.items
                .iter()
                .skip(app.offset)
                .take(visible_rows)
                .map(|(tag, c)| {
                    Row::new(vec![
                        Cell::from(c.to_string()).style(Style::default().fg(Color::Green)),
                        Cell::from(tag.as_str()),
                    ])
                })
                .collect()
        };

        let title = if app.is_search_mode() {
            format!("検索結果 ({} 件)", app.filtered_indices.len())
        } else {
            format!("先頭からプレビュー ({} 件)", app.items.len())
        };

        let table = Table::new(rows, [Constraint::Length(4), Constraint::Percentage(100)])
            .header(
                Row::new(vec![
                    Cell::from("文字").style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from("外字タグ").style(Style::default().add_modifier(Modifier::BOLD)),
                ])
                .style(Style::default().fg(Color::Yellow))
                .bottom_margin(1),
            )
            .block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(table, area);
    }
}
