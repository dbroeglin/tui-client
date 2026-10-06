# tui-client

A Ratatui client for the sibling `backend-api` repository.

## Run

Start the backend first, then:

```bash
cargo run
```

Set `DEMO_API_URL` to use a different backend URL.

| Key | Action |
| --- | --- |
| Up/Down or `j`/`k` | Select a task |
| Space or Enter | Toggle completion |
| `a` | Add a sample task |
| `d` | Delete the selected task |
| `r` | Refresh |
| `q` | Quit |
