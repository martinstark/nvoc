//! Domain layer for GPU operations and business logic
//!
//! This module provides high-level abstractions over raw NVML operations,
//! handling unit conversions, business logic, and domain-specific calculations.
//! It keeps the NVML wrapper purely focused on API bindings.

pub mod power;

pub use power::*;
