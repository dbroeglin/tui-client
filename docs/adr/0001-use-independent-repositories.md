# 0001: Use independent repositories in a multi-root workspace

- Status: Accepted
- Date: 2026-10-06
- Scope: `tui-client`, `backend-api`, `shared-models`

## Context

The demo must show how tooling can navigate and coordinate changes across
separately versioned Rust components. Combining the components in one
repository would simplify atomic changes, but it would not exercise
multi-repository discovery, source control, or dependency management.

Developers still need one convenient environment in which rust-analyzer can
load every crate and follow references across component boundaries.

## Decision

Maintain the TUI client, backend API, and shared models as independent
repositories. Open them together through the root `cld-demo.code-workspace`
multi-root workspace and list every crate in `rust-analyzer.linkedProjects`.

Coordinate cross-repository changes explicitly. Each affected repository keeps
its own history, validation, and release boundary.

## Consequences

- Source control operations and releases remain independent for each
  component.
- The workspace provides cross-repository navigation without creating a Cargo
  workspace or merging repository histories.
- A contract change can require multiple coordinated commits and cannot be
  made atomically across all repositories.
- Local path dependencies require the repositories to remain sibling
  directories during development.
- CI must validate each repository and should include an integration check when
  a change spans repository boundaries.
