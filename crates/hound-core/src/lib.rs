//! HOUND's Rust hunter core.
//!
//! HOUND-001A: validated HTTP observations and append-only evidence collection.
//! This crate must not depend on evaluator-only ground truth.

pub mod memory;
pub mod policy;
pub mod sensors;
pub mod truth;

/// Small, explicit error surface for this research slice.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
