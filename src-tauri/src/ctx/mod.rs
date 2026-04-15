//! Context Engine — structured retrieval + tool-output virtualization on top of
//! Claude Code hooks. Sidecar binary `glyphic-ctx` handles hook events; this
//! module exposes shared logic (DB, config, hook handlers) to both the sidecar
//! and the Tauri commands layer.

pub mod config;
pub mod db;
pub mod embed;
pub mod hook;
pub mod retrieve;
pub mod virtualize;
