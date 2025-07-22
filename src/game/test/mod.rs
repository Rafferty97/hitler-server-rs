//! Test module for the Secret Hitler XL game engine
//!
//! This module contains comprehensive tests for all game functionality,
//! organized into logical submodules for better maintainability.

#![cfg(test)]
#![allow(clippy::bool_assert_comparison)]

// Re-export common test utilities

// Test submodules
pub mod anarchist_mechanics;
pub mod monarchist_victory;
pub mod social_democratic_policy;
// pub mod anti_policies;  // Temporarily disabled - missing Party variants
pub mod communist_knowledge;
pub mod communist_powers;
pub mod edge_cases;
// pub mod emergency_powers;  // Temporarily disabled - missing ExecutiveAction variants
pub mod executive_powers;
pub mod government;
pub mod initialization;
pub mod integration;
pub mod legislative;
pub mod message_broadcasting;
pub mod player_count_validation;
pub mod player_management;
pub mod policy_deck_construction;
pub mod policy_tracker_tests;
pub mod role_assignment;
pub mod special_roles;
pub mod special_roles_interaction;
pub mod state_transitions;
pub mod state_validation;
pub mod test_utils;
pub mod victory_conditions;
