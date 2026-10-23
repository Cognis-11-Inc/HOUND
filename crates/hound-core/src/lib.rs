//! HOUND's Rust hunter core.
//!
//! Initial module boundaries only; hunting behavior is not implemented.
//! This crate must not depend on evaluator-only ground truth.

pub mod memory;
pub mod policy;
pub mod sensors;
pub mod truth;
