# tui-client

A Ratatui client for the sibling `backend-api` repository.

## Run

Start the backend first, then:

```bash
cargo run
```

Set `DEMO_API_URL` to use a different backend URL. It must be an HTTP or HTTPS
URL without a query string or fragment. Connections time out after 3 seconds,
and requests time out after 10 seconds.

## Architecture decisions

The [`docs/adr`](docs/adr/README.md) directory contains the architecture
decision records for the complete multi-repository workspace, including
`tui-client`, `backend-api`, and `shared-models`.

| Key | Action |
| --- | --- |
| Up/Down or `j`/`k` | Select a task |
| Space or Enter | Toggle completion |
| `a` | Open the new-task input |
| Tab | Switch between content and title while editing |
| Enter | Save a new task while editing |
| Esc | Cancel a new task while editing |
| `d` | Delete the selected task |
| `r` | Refresh |
| `q` | Quit |

The task list displays each task's title above its content.
