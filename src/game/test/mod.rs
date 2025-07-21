//! Test module for the Secret Hitler XL game engine
//!
//! This module contains comprehensive tests for all game functionality,
//! organized into logical submodules for better maintainability.

#![cfg(test)]
#![allow(clippy::bool_assert_comparison)]

// Re-export common test utilities
pub use test_utils::*;

// Test submodules
pub mod communist_powers;
pub mod edge_cases;
pub mod executive_powers;
pub mod government;
pub mod initialization;
pub mod integration;
pub mod legislative;
pub mod message_broadcasting;
pub mod player_management;
pub mod special_roles;
pub mod state_transitions;
pub mod state_validation;
pub mod test_utils;
pub mod victory_conditions;
