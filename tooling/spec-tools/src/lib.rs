//! The spec tools behind `tooling/up spec-id` and `tooling/up spec-check` (spec/README.md): new date-based ids for
//! log entries (L) and interpretation rows (J), and the consistency check of `spec/log/`, `spec/interpretations.md`
//! and `spec/current/`.

pub mod checker;
pub mod ids;
pub mod model;
pub mod sources;
pub mod time;
