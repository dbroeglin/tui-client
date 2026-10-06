# 0002: Share API contracts through a Rust crate

- Status: Accepted
- Date: 2026-10-06
- Scope: `tui-client`, `backend-api`, `shared-models`

## Context

The client and server exchange task, health, request, and error payloads. If
each application defines those payloads independently, their JSON contracts
can drift while both repositories continue to compile.

The applications are both written in Rust and can reuse strongly typed data
structures. The demo also needs contract changes to be discoverable across
repository boundaries.

## Decision

Define transport data structures and validation that belongs to those
structures in the `shared-models` crate. Derive Serde serialization and
deserialization there, and consume the crate from both applications through a
local path dependency during development.

Keep application behavior out of `shared-models`. HTTP routing and persistence
belong to `backend-api`; rendering, input handling, and network orchestration
belong to `tui-client`.

## Consequences

- Client and server compile against the same payload shapes.
- Contract changes become explicit coordinated changes across repositories.
- Validation intrinsic to a request, such as normalization of user-provided
  fields, has one implementation.
- A published or otherwise pinned crate version will be required when the
  applications are built without sibling repository checkouts.
- Shared Rust types couple the client and server release process more tightly
  than a language-neutral schema would.
- If non-Rust consumers are introduced, this decision should be revisited in
  favor of an OpenAPI or other language-neutral contract as the source of
  truth.
