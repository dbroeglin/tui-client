use std::{
    io::{self, Stdout},
    time::Duration,
};

use anyhow::{Context, Result, anyhow, bail};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use reqwest::{Client, RequestBuilder, Response, Url};
use shared_models::{CreateTask, ErrorResponse, Task};

const DEFAULT_API_URL: &str = "http://127.0.0.1:3000";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputField {
    Content,
    Title,
}

struct TaskInput {
    content: String,
    title: String,
    field: InputField,
}

impl TaskInput {
    fn new() -> Self {
        Self {
            content: String::new(),
            title: String::new(),
            field: InputField::Content,
        }
    }

    fn active(&self) -> &str {
        match self.field {
            InputField::Content => &self.content,
            InputField::Title => &self.title,
        }
    }

    fn active_mut(&mut self) -> &mut String {
        match self.field {
            InputField::Content => &mut self.content,
            InputField::Title => &mut self.title,
        }
    }

    fn toggle_field(&mut self) {
        self.field = match self.field {
            InputField::Content => InputField::Title,
            InputField::Title => InputField::Content,
        };
    }
}

struct ApiClient {
    base_url: Url,
    http: Client,
}

impl ApiClient {
    fn new(base_url: String) -> Result<Self> {
        let mut base_url = Url::parse(&base_url).context("DEMO_API_URL is not a valid URL")?;
        if !matches!(base_url.scheme(), "http" | "https") {
            bail!("DEMO_API_URL must use http or https");
        }
        if base_url.cannot_be_a_base() {
            bail!("DEMO_API_URL cannot be used as a base URL");
        }
        if base_url.query().is_some() || base_url.fragment().is_some() {
            bail!("DEMO_API_URL must not contain a query string or fragment");
        }
        let path = format!("{}/", base_url.path().trim_end_matches('/'));
        base_url.set_path(&path);

        let http = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .context("failed to configure HTTP client")?;

        Ok(Self { base_url, http })
    }

    fn endpoint(&self, path: &str) -> Result<Url> {
        self.base_url
            .join(path)
            .with_context(|| format!("failed to construct API endpoint for {path}"))
    }

    async fn send(&self, request: RequestBuilder, operation: &str) -> Result<Response> {
        request.send().await.map_err(|error| {
            if error.is_timeout() {
                anyhow!("timed out while {operation}")
            } else if error.is_connect() {
                anyhow!("could not connect to the backend while {operation}: {error}")
            } else {
                anyhow!("HTTP failure while {operation}: {error}")
            }
        })
    }

    async fn require_success(&self, response: Response, operation: &str) -> Result<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        match response.json::<ErrorResponse>().await {
            Ok(error) => bail!("backend rejected {operation} ({status}): {}", error.message),
            Err(error) => bail!(
                "backend rejected {operation} ({status}) and returned an invalid error response: {error}"
            ),
        }
    }

    async fn tasks(&self) -> Result<Vec<Task>> {
        let response = self
            .send(self.http.get(self.endpoint("api/tasks")?), "loading tasks")
            .await?;
        self.require_success(response, "task list request")
            .await?
            .json()
            .await
            .context("backend returned an invalid task list")
    }

    async fn create(&self, title: String, content: String) -> Result<Task> {
        let response = self
            .send(
                self.http
                    .post(self.endpoint("api/tasks")?)
                    .json(&CreateTask { title, content }),
                "creating a task",
            )
            .await?;
        self.require_success(response, "task creation")
            .await?
            .json()
            .await
            .context("backend returned an invalid task")
    }

    async fn toggle(&self, id: u64) -> Result<Task> {
        let response = self
            .send(
                self.http.patch(self.endpoint(&format!("api/tasks/{id}"))?),
                "updating a task",
            )
            .await?;
        self.require_success(response, "task update")
            .await?
            .json()
            .await
            .context("backend returned an invalid task")
    }

    async fn delete(&self, id: u64) -> Result<()> {
        let response = self
            .send(
                self.http.delete(self.endpoint(&format!("api/tasks/{id}"))?),
                "deleting a task",
            )
            .await?;
        self.require_success(response, "task deletion").await?;
        Ok(())
    }
}

struct App {
    api: ApiClient,
    tasks: Vec<Task>,
    selection: ListState,
    message: String,
    task_input: Option<TaskInput>,
}

impl App {
    fn new(api: ApiClient) -> Self {
        Self {
            api,
            tasks: Vec::new(),
            selection: ListState::default(),
            message: "Press r to load tasks".into(),
            task_input: None,
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

    fn begin_add(&mut self) {
        self.task_input = Some(TaskInput::new());
        self.message = "Enter task content and title; Tab switches fields".into();
    }

    fn cancel_add(&mut self) {
        self.task_input = None;
        self.message = "Task creation cancelled".into();
    }

    fn push_input(&mut self, character: char) {
        if let Some(input) = &mut self.task_input {
            input.active_mut().push(character);
        }
    }

    fn pop_input(&mut self) {
        if let Some(input) = &mut self.task_input {
            input.active_mut().pop();
        }
    }

    async fn handle_add_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            KeyCode::Esc => self.cancel_add(),
            KeyCode::Enter | KeyCode::Char('\n' | '\r') => self.submit_add().await,
            KeyCode::Tab | KeyCode::BackTab => {
                if let Some(input) = &mut self.task_input {
                    input.toggle_field();
                }
            }
            KeyCode::Backspace => self.pop_input(),
            KeyCode::Char(character)
                if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.push_input(character);
            }
            _ => {}
        }
    }

    async fn submit_add(&mut self) {
        let Some(input) = &self.task_input else {
            return;
        };
        let content = input.content.trim();
        if content.is_empty() {
            self.message = "Task content cannot be blank".into();
            return;
        }
        let title = input.title.trim();
        if title.is_empty() {
            self.message = "Task title cannot be blank".into();
            return;
        }
        let content = content.to_owned();
        let title = title.to_owned();

        match self.api.create(title, content).await {
            Ok(task) => {
                self.tasks.push(task);
                self.selection.select(Some(self.tasks.len() - 1));
                self.task_input = None;
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
    let api = ApiClient::new(api_url)?;
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, App::new(api)).await;
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
        if key.kind == KeyEventKind::Release {
            continue;
        }
        if app.task_input.is_some() {
            app.handle_add_key(key.code, key.modifiers).await;
            continue;
        }
        match key.code {
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Char('r') => app.refresh().await,
            KeyCode::Char('a') => app.begin_add(),
            KeyCode::Char(' ') | KeyCode::Enter => app.toggle_selected().await,
            KeyCode::Char('d') | KeyCode::Delete => app.delete_selected().await,
            KeyCode::Down | KeyCode::Char('j') => app.select_next(),
            KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
            _ => {}
        }
    }
}

fn render(frame: &mut ratatui::Frame, app: &mut App) {
    let constraints = if app.task_input.is_some() {
        vec![
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Length(4),
        ]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(4),
        ]
    };
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
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
        ListItem::new(vec![
            Line::from(format!("{marker} {}", task.title)),
            Line::from(Span::styled(
                task.content.as_str(),
                Style::default().add_modifier(Modifier::ITALIC),
            )),
        ])
        .style(style)
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

    if let Some(input) = &app.task_input {
        let input_area = areas[3];
        let content_style = if input.field == InputField::Content {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let title_style = if input.field == InputField::Title {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::raw("Content: "),
                    Span::styled(input.content.as_str(), content_style),
                ]),
                Line::from(vec![
                    Span::raw("Title:   "),
                    Span::styled(input.title.as_str(), title_style),
                ]),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("New task (Tab switches, Enter saves, Esc cancels)"),
            ),
            input_area,
        );
        let max_offset = input_area.width.saturating_sub(11);
        let input_offset = u16::try_from(input.active().chars().count())
            .unwrap_or(u16::MAX)
            .min(max_offset);
        let cursor_y = match input.field {
            InputField::Content => input_area.y + 1,
            InputField::Title => input_area.y + 2,
        };
        frame.set_cursor_position((input_area.x + 10 + input_offset, cursor_y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::new(ApiClient::new(DEFAULT_API_URL.into()).unwrap())
    }

    #[test]
    fn api_client_normalizes_the_base_url() {
        let client = ApiClient::new("http://localhost:3000/demo".into()).unwrap();

        assert_eq!(client.base_url.as_str(), "http://localhost:3000/demo/");
        assert_eq!(
            client.endpoint("api/tasks").unwrap().as_str(),
            "http://localhost:3000/demo/api/tasks"
        );
    }

    #[test]
    fn api_client_rejects_invalid_base_urls() {
        let invalid_url = ApiClient::new("not a URL".into()).err().unwrap();
        let unsupported_scheme = ApiClient::new("ftp://localhost/tasks".into())
            .err()
            .unwrap();
        let query = ApiClient::new("http://localhost:3000?tenant=demo".into())
            .err()
            .unwrap();

        assert!(invalid_url.to_string().contains("not a valid URL"));
        assert!(unsupported_scheme.to_string().contains("http or https"));
        assert!(query.to_string().contains("query string or fragment"));
    }

    #[test]
    fn add_mode_collects_and_cancels_input() {
        let mut app = app();

        app.begin_add();
        app.push_input('D');
        app.push_input('o');
        app.pop_input();

        assert_eq!(
            app.task_input.as_ref().map(|input| input.content.as_str()),
            Some("D")
        );
        app.cancel_add();
        assert!(app.task_input.is_none());
    }

    #[tokio::test]
    async fn blank_content_stays_in_add_mode() {
        let mut app = app();
        app.begin_add();
        app.push_input(' ');

        app.handle_add_key(KeyCode::Enter, KeyModifiers::NONE).await;

        assert_eq!(
            app.task_input.as_ref().map(|input| input.content.as_str()),
            Some(" ")
        );
        assert_eq!(app.message, "Task content cannot be blank");
    }

    #[tokio::test]
    async fn blank_title_stays_in_add_mode() {
        let mut app = app();
        app.begin_add();
        app.push_input('C');
        app.handle_add_key(KeyCode::Tab, KeyModifiers::NONE).await;

        app.handle_add_key(KeyCode::Enter, KeyModifiers::NONE).await;

        assert_eq!(app.message, "Task title cannot be blank");
    }
}
