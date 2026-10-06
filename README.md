# tui-client

A Ratatui task dashboard for the
[`backend-api`](https://github.com/dbroeglin/backend-api) service. The client
lists, creates, updates, and deletes tasks through the shared HTTP API.

## Prerequisites

- A current stable [Rust toolchain](https://www.rust-lang.org/tools/install)
  with support for Rust 2024 edition
- Local checkouts of
  [`backend-api`](https://github.com/dbroeglin/backend-api) and
  [`shared-models`](https://github.com/dbroeglin/shared-models)

This repository uses a local path dependency on `../shared-models`. Keep the
three repositories in sibling directories:

```text
demo/
├── backend-api/
├── shared-models/
└── tui-client/
```

## Run

1. Start the backend in its repository:

   ```bash
   cd ../backend-api
   cargo run
   ```

   The backend listens on `http://127.0.0.1:3000`.

2. In another terminal, start this client:

   ```bash
   cargo run
   ```

The client loads tasks when it starts. API failures are displayed in the
status area and can be retried with `r`.

## Configuration

The client reads the backend base URL from `DEMO_API_URL` and defaults to
`http://127.0.0.1:3000`. The URL:

- must use HTTP or HTTPS;
- may include a path prefix;
- must not include a query string or fragment.

For example:

```powershell
$env:DEMO_API_URL = "http://localhost:3000"
cargo run
```

```bash
DEMO_API_URL=http://localhost:3000 cargo run
```

Connections time out after 3 seconds, and complete requests time out after
10 seconds.

## Controls

| Key | Action |
| --- | --- |
| Up/Down or `j`/`k` | Select a task |
| Space or Enter | Toggle completion |
| `a` | Open the new-task input |
| Tab or Shift+Tab | Switch between content and title while editing |
| Enter | Save a new task while editing |
| Esc | Cancel a new task while editing |
| `d` or Delete | Delete the selected task |
| `r` | Refresh |
| `q` | Quit |

The task list displays each task's content above its title.

## Development

Run the repository checks before submitting a change:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Architecture decisions

The [`docs/adr`](docs/adr/README.md) directory contains the architecture
decision records for the complete multi-repository workspace, including this
client, the backend API, and the shared models crate.
