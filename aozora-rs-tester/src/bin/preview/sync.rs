use std::io;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::thread;

use aozora_rs_tester::{
    GitSyncProgress, MapCacheProgress, sync_repository, update_map_with_progress,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use super::Screen;

#[derive(Debug, Clone)]
enum SyncMessage {
    Git(GitSyncProgress),
    Map(MapCacheProgress),
    AllDone,
    Error(String),
}

struct SyncState {
    messages: Vec<String>,
    current_step: String,
    is_complete: bool,
    has_error: bool,
    progress_percent: u16,
}

impl SyncState {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            current_step: "準備中...".to_string(),
            is_complete: false,
            has_error: false,
            progress_percent: 0,
        }
    }

    fn handle_message(&mut self, msg: SyncMessage) {
        match msg {
            SyncMessage::Git(p) => {
                let text = match p {
                    GitSyncProgress::Checking => {
                        self.progress_percent = 5;
                        "リポジトリを確認中..."
                    }
                    GitSyncProgress::Cloning => {
                        self.progress_percent = 10;
                        "クローン中..."
                    }
                    GitSyncProgress::Pulling => {
                        self.progress_percent = 10;
                        "最新版を取得中..."
                    }
                    GitSyncProgress::Done => {
                        self.progress_percent = 40;
                        "Git同期完了"
                    }
                    GitSyncProgress::Error(ref e) => {
                        self.has_error = true;
                        self.messages.push(format!("エラー: {}", e));
                        "エラー発生"
                    }
                };
                self.current_step = text.to_string();
                if !matches!(p, GitSyncProgress::Error(_)) {
                    self.messages.push(format!("[Git] {}", text));
                }
            }
            SyncMessage::Map(p) => {
                let text = match p {
                    MapCacheProgress::CheckingCache => {
                        self.progress_percent = 45;
                        "キャッシュを確認中..."
                    }
                    MapCacheProgress::CacheFound => {
                        self.progress_percent = 50;
                        "キャッシュが見つかりました"
                    }
                    MapCacheProgress::CacheOutdated => {
                        self.progress_percent = 55;
                        "キャッシュが古いです - 更新中"
                    }
                    MapCacheProgress::CacheUpToDate => {
                        self.progress_percent = 90;
                        "キャッシュは最新です"
                    }
                    MapCacheProgress::CacheNotFound => {
                        self.progress_percent = 55;
                        "キャッシュなし - 作成中"
                    }
                    MapCacheProgress::GeneratingMap => {
                        self.progress_percent = 60;
                        "マップを生成中..."
                    }
                    MapCacheProgress::SavingCache => {
                        self.progress_percent = 85;
                        "キャッシュを保存中..."
                    }
                    MapCacheProgress::Done => {
                        self.progress_percent = 95;
                        "マップ更新完了"
                    }
                };
                self.current_step = text.to_string();
                self.messages.push(format!("[Map] {}", text));
            }
            SyncMessage::AllDone => {
                self.is_complete = true;
                self.progress_percent = 100;
                self.current_step = "すべて完了！".to_string();
                self.messages.push("同期が正常に完了しました".to_string());
            }
            SyncMessage::Error(e) => {
                self.has_error = true;
                self.current_step = "エラー".to_string();
                self.messages.push(format!("エラー: {}", e));
            }
        }
    }
}

pub fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<Screen> {
    let mut state = SyncState::new();

    // Create channel for progress updates
    let (tx, rx): (_, Receiver<SyncMessage>) = channel();

    // Spawn sync thread
    let current_dir = std::env::current_dir()?;
    let tx_git = tx.clone();
    let tx_map = tx.clone();
    thread::spawn(move || {
        // Git sync
        if let Err(e) = sync_repository(&current_dir, |p| {
            let _ = tx_git.send(SyncMessage::Git(p));
        }) {
            let _ = tx.send(SyncMessage::Error(e.to_string()));
            return;
        }

        // Map update
        if let Err(e) = update_map_with_progress(&current_dir, |p| {
            let _ = tx_map.send(SyncMessage::Map(p));
        }) {
            let _ = tx.send(SyncMessage::Error(e.to_string()));
            return;
        }

        let _ = tx.send(SyncMessage::AllDone);
    });

    loop {
        // Process any pending messages
        loop {
            match rx.try_recv() {
                Ok(msg) => state.handle_message(msg),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if !state.is_complete && !state.has_error {
                        state.handle_message(SyncMessage::Error(
                            "同期スレッドが予期せず終了しました".to_string(),
                        ));
                    }
                    break;
                }
            }
        }

        terminal.draw(|f| ui(f, &state))?;

        // Non-blocking event check
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => return Ok(Screen::Home),
                        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(Screen::Exit);
                        }
                        KeyCode::Enter if state.is_complete || state.has_error => {
                            return Ok(Screen::Home);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, state: &SyncState) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Length(3), // Progress bar
        Constraint::Min(1),    // Log messages
        Constraint::Length(3), // Footer
    ])
    .split(f.area());

    // Header
    let status_color = if state.has_error {
        Color::Red
    } else if state.is_complete {
        Color::Green
    } else {
        Color::Cyan
    };

    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "青空文庫 同期",
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            &state.current_step,
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::BOTTOM))
    .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Progress bar
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" 進捗 "))
        .gauge_style(Style::default().fg(status_color))
        .percent(state.progress_percent)
        .label(format!("{}%", state.progress_percent));
    f.render_widget(gauge, chunks[1]);

    // Log messages
    let items: Vec<ListItem> = state
        .messages
        .iter()
        .rev()
        .take((chunks[2].height.saturating_sub(2)) as usize)
        .map(|m| {
            let style = if m.contains("エラー") {
                Style::default().fg(Color::Red)
            } else if m.contains("完了") {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(m.as_str(), style)))
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" ログ "));
    f.render_widget(list, chunks[2]);

    // Footer
    let footer_text = if state.is_complete || state.has_error {
        "Enter: 戻る  Esc: ホームへ"
    } else {
        "同期中... Esc: キャンセル"
    };
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[3]);
}
