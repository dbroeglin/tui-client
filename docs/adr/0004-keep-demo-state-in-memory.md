# 0004: Keep demo task state in memory

- Status: Accepted
- Date: 2026-10-06
- Scope: `backend-api`

## Context

The backend exists to demonstrate a multi-repository client, API, and shared
contract workflow. Introducing a database would add setup, migrations,
credentials, and operational concerns that are not required to demonstrate
that workflow.

The service still needs state shared safely among concurrent Axum handlers.

## Decision

Store tasks in a process-local `Vec<Task>` protected by `Arc<RwLock<_>>`.
Seed a small task set when the application starts. Allocate an identifier as
one greater than the largest current identifier.

Treat poisoned lock access as an internal server error rather than hiding it or
returning stale data.

## Consequences

- The backend runs without external infrastructure.
- Reads and writes are safe across concurrent request handlers.
- All changes are lost when the process restarts.
- Multiple backend instances do not share state and can allocate duplicate
  identifiers.
- The lock and linear scans are appropriate only for the small demo data set.
- Persistence, horizontal scaling, querying, and durable identifier generation
  require superseding this ADR with a database-backed design.
