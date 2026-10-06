# Architecture decision records

This directory is the central architecture decision log for the complete demo
workspace:

- `tui-client`
- `backend-api`
- `shared-models`

The records live in `tui-client` so developers have one discoverable decision
log while working across the three independent repositories. A decision can
apply to one component or to the integration among components; each record
states its scope.

## Status values

- **Proposed**: under discussion and not yet binding.
- **Accepted**: the current architectural direction.
- **Deprecated**: retained for history but no longer recommended.
- **Superseded**: replaced by another ADR.

## Decisions

| ADR | Status | Scope |
| --- | --- | --- |
| [0001: Use independent repositories in a multi-root workspace](0001-use-independent-repositories.md) | Accepted | All repositories |
| [0002: Share API contracts through a Rust crate](0002-share-api-contracts-through-rust-crate.md) | Accepted | All repositories |
| [0003: Use HTTP and JSON between the TUI and backend](0003-use-http-json-api.md) | Accepted | `tui-client`, `backend-api` |
| [0004: Keep demo task state in memory](0004-keep-demo-state-in-memory.md) | Accepted | `backend-api` |
| [0005: Build the client as an asynchronous terminal UI](0005-build-an-async-terminal-ui.md) | Accepted | `tui-client` |

## Adding or changing a decision

1. Copy the structure of an existing record.
2. Assign the next four-digit number; never reuse a number.
3. Describe the context, decision, consequences, and affected repositories.
4. Add the record to the table above.
5. Do not rewrite an accepted decision after the architecture changes. Add a
   new ADR and mark the old record as superseded.
