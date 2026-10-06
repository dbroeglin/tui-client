# 0003: Use HTTP and JSON between the TUI and backend

- Status: Accepted
- Date: 2026-10-06
- Scope: `tui-client`, `backend-api`

## Context

The terminal client and backend are separate processes and repositories. They
need a simple boundary that is easy to run locally, inspect manually, and
extend for demo scenarios. The transport should support typed payloads from
`shared-models` without requiring protocol-specific code generation.

## Decision

Expose task operations from `backend-api` as a JSON HTTP API built with Axum.
Call that API from `tui-client` with Reqwest.

Use resource-oriented routes under `/api/tasks`, plus `/health` for service
health. Treat non-success HTTP status codes and invalid response bodies as
errors. Configure the client base URL with `DEMO_API_URL`, defaulting to
`http://127.0.0.1:3000`.

## Consequences

- The boundary can be exercised with common HTTP tools and can support clients
  written in other languages.
- Serde-backed shared models map directly to request and response bodies.
- The client and backend can be started, stopped, and deployed independently.
- API compatibility must be maintained across independent releases.
- The current local default is convenient for development but deployment must
  provide an explicit endpoint.
- Authentication, transport security, API versioning, retry policy, and
  production CORS restrictions are deferred. They must be decided before the
  API is exposed beyond this local demo.
