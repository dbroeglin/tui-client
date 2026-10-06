# 0005: Build the client as an asynchronous terminal UI

- Status: Accepted
- Date: 2026-10-06
- Scope: `tui-client`

## Context

The client must provide an interactive task dashboard that runs in a terminal
and communicates with the backend without introducing a graphical runtime.
It needs keyboard input, selection state, forms, rendering, and terminal
restoration when the application exits.

Network operations are asynchronous, while terminal input is polled from the
same application loop.

## Decision

Build the UI with Ratatui and Crossterm on a Tokio runtime. Keep task data,
selection, user messages, and form state in a single application state object.
Use Reqwest for asynchronous backend calls.

Run the terminal in raw mode on an alternate screen. Restore raw mode, the
original screen, and cursor visibility before returning from the application.
Display API failures in the UI instead of treating failed operations as
successful.

## Consequences

- The client remains portable across terminals supported by Crossterm.
- A single state owner makes rendering and selection repair predictable.
- Async HTTP calls fit the Tokio-based backend and Reqwest client.
- Awaiting an HTTP operation in the input loop temporarily pauses input and
  rendering. This is acceptable for the local demo but should be replaced by
  background requests and explicit loading state if latency becomes material.
- Terminal setup and cleanup must remain paired whenever lifecycle behavior
  changes.
- The UI depends on keyboard interaction; accessibility and non-interactive
  automation require separate interfaces.
