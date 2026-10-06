use std::{
    io::{self, Stdout},
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use reqwest::Client;
use shared_models::{CreateTask, Task};

const DEFAULT_API_URL: &str = "http://127.0.0.1:3000";

struct ApiClient {
    base_url: String,
    http: Client,
}

impl ApiClient {
    fn new(base_url: String) -> Self {
        Self {
            base_url,
            http: Client::new(),
        }
    }

    async fn tasks(&self) -> Result<Vec<Task>> {
        self.http
            .get(format!("{}/api/tasks", self.base_url))
            .send()
            .await
            .context("failed to connect to backend API")?
            .error_for_status()
            .context("backend rejected task list request")?
            .json()
            .await
            .context("backend returned an invalid task list")
    }

    async fn create(&self, title: String) -> Result<Task> {
        self.http
            .post(format!("{}/api/tasks", self.base_url))
            .json(&CreateTask { title })
            .send()
            .await
            .context("failed to connect to backend API")?
            .error_for_status()
            .context("backend rejected task creation")?
            .json()
            .await
            .context("backend returned an invalid task")
    }

    async fn toggle(&self, id: u64) -> Result<Task> {
        self.http
            .patch(format!("{}/api/tasks/{id}", self.base_url))
            .send()
            .await
            .context("failed to connect to backend API")?
            .error_for_status()
            .context("backend rejected task update")?
            .json()
            .await
            .context("backend returned an invalid task")
    }

    async fn delete(&self, id: u64) -> Result<()> {
        self.http
            .delete(format!("{}/api/tasks/{id}", self.base_url))
            .send()
            .await
            .context("failed to connect to backend API")?
            .error_for_status()
            .context("backend rejected task deletion")?;
        Ok(())
    }
}

struct App {
    api: ApiClient,
    tasks: Vec<Task>,
    selection: ListState,
    message: String,
    sample_number: usize,
}

impl App {
    fn new(api: ApiClient) -> Self {
        Self {
            api,
            tasks: Vec::new(),
            selection: ListState::default(),
            message: "Press r to load tasks".into(),
            sample_number: 1,
        }
    }

    async fn refresh(&mut self) {
        match self.api.tasks().await {
            Ok(tasks) => {
                self.tasks = tasks;
                self.repair_selection();
                self.message = format!("Loaded {} tasks", self.tasks.len());
            }
            Err(error) => self.message = format!("{error:#}"),
        }
    }

    async fn add_sample(&mut self) {
        let title = format!("Copilot demo task {}", self.sample_number);
        match self.api.create(title).await {
            Ok(task) => {
                self.tasks.push(task);
                self.selection.select(Some(self.tasks.len() - 1));
                self.sample_number += 1;
                self.message = "Task created".into();
            }
            Err(error) => self.message = format!("{error:#}"),
        }
    }

    async fn toggle_selected(&mut self) {
        let Some(index) = self.selection.selected() else {
            self.message = "No task selected".into();
            return;
        };
        let id = self.tasks[index].id;
        match self.api.toggle(id).await {
            Ok(task) => {
                self.tasks[index] = task;
                self.message = "Task updated".into();
            }
            Err(error) => self.message = format!("{error:#}"),
        }
    }

    async fn delete_selected(&mut self) {
        let Some(index) = self.selection.selected() else {
            self.message = "No task selected".into();
            return;
        };
        let id = self.tasks[index].id;
        match self.api.delete(id).await {
            Ok(()) => {
                self.tasks.remove(index);
                self.repair_selection();
                self.message = "Task deleted".into();
            }
            Err(error) => self.message = format!("{error:#}"),
        }
    }

    fn select_next(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let next = self
            .selection
            .selected()
            .map_or(0, |index| (index + 1) % self.tasks.len());
        self.selection.select(Some(next));
    }

    fn select_previous(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let previous = self.selection.selected().map_or(0, |index| {
            if index == 0 {
                self.tasks.len() - 1
            } else {
                index - 1
            }
        });
        self.selection.select(Some(previous));
    }

    fn repair_selection(&mut self) {
        match (self.tasks.is_empty(), self.selection.selected()) {
            (true, _) => self.selection.select(None),
            (false, Some(index)) if index >= self.tasks.len() => {
                self.selection.select(Some(self.tasks.len() - 1));
            }
            (false, None) => self.selection.select(Some(0)),
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let api_url = std::env::var("DEMO_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.into());
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, App::new(ApiClient::new(api_url))).await;
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().context("failed to enable terminal raw mode")?;
    let mut stdout = io::stdout();
    if let Err(error) = execute!(stdout, EnterAlternateScreen) {
        disable_raw_mode().ok();
        return Err(error).context("failed to enter alternate screen");
    }
    Terminal::new(CrosstermBackend::new(stdout)).context("failed to initialize terminal")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("failed to disable terminal raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("failed to leave alternate screen")?;
    terminal.show_cursor().context("failed to restore cursor")
}

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> Result<()> {
    app.refresh().await;
    loop {
        terminal.draw(|frame| render(frame, &mut app))?;
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Char('r') => app.refresh().await,
            KeyCode::Char('a') => app.add_sample().await,
            KeyCode::Char(' ') | KeyCode::Enter => app.toggle_selected().await,
            KeyCode::Char('d') | KeyCode::Delete => app.delete_selected().await,
            KeyCode::Down | KeyCode::Char('j') => app.select_next(),
            KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
            _ => {}
        }
    }
}

fn render(frame: &mut ratatui::Frame, app: &mut App) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(3),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("Multi-Repository Task Dashboard")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL)),
        areas[0],
    );

    let items = app.tasks.iter().map(|task| {
        let marker = if task.completed { "[x]" } else { "[ ]" };
        let style = if task.completed {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };
        ListItem::new(Line::from(format!("{marker} {}", task.title))).style(style)
    });
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, areas[1], &mut app.selection);

    let footer = format!(
        "↑/↓ select  Space toggle  a add  d delete  r refresh  q quit\n{}",
        app.message
    );
    frame.render_widget(
        Paragraph::new(footer).block(Block::default().borders(Borders::ALL)),
        areas[2],
    );
}
