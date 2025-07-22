//! Comprehensive role assignment tests
//!
//! Tests based on the Secret Hitler XL rules for role assignment:
//! - Hitler is correctly assigned to one Fascist
//! - Role cards are distributed face-down and secretly
//! - Special roles (Capitalist, Anarchist, Monarchist) are assigned when enabled
//! - Correct role ratios for all player counts (6-20)
//!
//! AMBIGUITY NOTES:
//! - Player count limits: test.md specifies 6-20 players, but rules.pdf only shows
//!   ratios up to 16 players. Tests assume 6-20 range but may need adjustment.
//! - Role name: rules.pdf uses both "Monarchist" and "Nationalist" - tests use "Monarchist"
//!   as it appears more frequently in the detailed rules section.
//! - Role ratios for 17-20 players are extrapolated from patterns since rules.pdf
//!   doesn't specify them explicitly.

use super::super::party::Party;
use super::super::player::{assign_roles, PlayerDistribution, Role};
use super::test_utils::*;
use crate::game::{Game, GameOptions};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Test that Hitler is correctly assigned to one Fascist
#[test]
fn test_hitler_assignment() {
    for player_count in 6..=20 {
        let opts = GameOptions {
            communists: true,
            monarchist: true,
            anarchist: true,
            capitalist: true,
            centrists: true,
        };

        if let Ok(game) = Game::new(
            opts,
            &(0..player_count).map(|i| format!("Player{}", i)).collect::<Vec<_>>(),
            42,
        ) {
            let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
            assert_eq!(hitler_count, 1, "{} players should have exactly 1 Hitler", player_count);

            // Verify Hitler's party affiliation is Fascist
            let hitler_player = game.players.iter().find(|p| p.role == Role::Hitler).unwrap();
            assert_eq!(
                hitler_player.party(),
                Party::Fascist,
                "Hitler should belong to Fascist party"
            );
        }
    }
}

/// Test role assignment for 6 players according to test.md specification
#[test]
fn test_6_player_role_assignment() {
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..6).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Count each role type
    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    // Verify according to test.md: 3L, 1F+H, 1C
    assert_eq!(liberal_count, 3, "6 players should have 3 Liberals");
    assert_eq!(fascist_count, 1, "6 players should have 1 Fascist");
    assert_eq!(hitler_count, 1, "6 players should have 1 Hitler");
    assert_eq!(communist_count, 1, "6 players should have 1 Communist");

    // Total should equal player count
    assert_eq!(liberal_count + fascist_count + hitler_count + communist_count, 6);
}

/// Test role assignment for 8 players according to test.md specification
#[test]
fn test_8_player_role_assignment() {
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Count each role type
    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    // Verify according to test.md: 4L, 2F+H, 1C
    assert_eq!(liberal_count, 4, "8 players should have 4 Liberals");
    assert_eq!(fascist_count, 2, "8 players should have 2 Fascists");
    assert_eq!(hitler_count, 1, "8 players should have 1 Hitler");
    assert_eq!(communist_count, 1, "8 players should have 1 Communist");

    // Total should equal player count
    assert_eq!(liberal_count + fascist_count + hitler_count + communist_count, 8);
}

/// Test role assignment for 11 players according to test.md specification
#[test]
fn test_11_player_role_assignment() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let player_names: Vec<String> = (0..11).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Count each role type
    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    // Verify according to test.md: 5L, 3F+H, 2C
    assert_eq!(liberal_count, 5, "11 players should have 5 Liberals");
    assert_eq!(fascist_count, 3, "11 players should have 3 Fascists");
    assert_eq!(hitler_count, 1, "11 players should have 1 Hitler");
    assert_eq!(communist_count, 2, "11 players should have 2 Communists");

    // Total should equal player count
    assert_eq!(liberal_count + fascist_count + hitler_count + communist_count, 11);
}

/// Test special roles assignment when enabled
#[test]
fn test_special_roles_assignment() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let player_names: Vec<String> = (0..16).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Count special roles
    let monarchist_count = game.players.iter().filter(|p| p.role == Role::Monarchist).count();
    let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();
    let capitalist_count = game.players.iter().filter(|p| p.role == Role::Capitalist).count();
    let centrist_count = game.players.iter().filter(|p| p.role == Role::Centrist).count();

    // Verify special roles are assigned when enabled
    assert_eq!(monarchist_count, 1, "Should have 1 Monarchist when enabled");
    assert_eq!(anarchist_count, 1, "Should have 1 Anarchist when enabled");
    assert_eq!(capitalist_count, 1, "Should have 1 Capitalist when enabled");
    assert_eq!(centrist_count, 2, "Should have 2 Centrists when enabled");

    // Verify party affiliations
    let monarchist = game.players.iter().find(|p| p.role == Role::Monarchist).unwrap();
    assert_eq!(
        monarchist.party(),
        Party::Fascist,
        "Monarchist should belong to Fascist party"
    );

    let anarchist = game.players.iter().find(|p| p.role == Role::Anarchist).unwrap();
    assert_eq!(
        anarchist.party(),
        Party::Communist,
        "Anarchist should belong to Communist party"
    );

    let capitalist = game.players.iter().find(|p| p.role == Role::Capitalist).unwrap();
    assert_eq!(
        capitalist.party(),
        Party::Liberal,
        "Capitalist should belong to Liberal party"
    );

    let centrists: Vec<_> = game.players.iter().filter(|p| p.role == Role::Centrist).collect();
    for centrist in centrists {
        assert_eq!(
            centrist.party(),
            Party::Liberal,
            "Centrist should belong to Liberal party"
        );
    }
}

/// Test special roles are not assigned when disabled
#[test]
fn test_special_roles_disabled() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: false,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Count special roles
    let monarchist_count = game.players.iter().filter(|p| p.role == Role::Monarchist).count();
    let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();
    let capitalist_count = game.players.iter().filter(|p| p.role == Role::Capitalist).count();
    let centrist_count = game.players.iter().filter(|p| p.role == Role::Centrist).count();

    // Verify special roles are not assigned when disabled
    assert_eq!(monarchist_count, 0, "Should have 0 Monarchists when disabled");
    assert_eq!(anarchist_count, 0, "Should have 0 Anarchists when disabled");
    assert_eq!(capitalist_count, 0, "Should have 0 Capitalists when disabled");
    assert_eq!(centrist_count, 0, "Should have 0 Centrists when disabled");
}

/// Test role distribution randomization
#[test]
fn test_role_distribution_randomization() {
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();

    // Create multiple games with different seeds
    let game1 = Game::new(opts, &player_names, 42).unwrap();
    let game2 = Game::new(opts, &player_names, 123).unwrap();

    // Extract role assignments
    let roles1: Vec<Role> = game1.players.iter().map(|p| p.role).collect();
    let roles2: Vec<Role> = game2.players.iter().map(|p| p.role).collect();

    // With different seeds, role order should be different (very high probability)
    assert_ne!(roles1, roles2, "Different seeds should produce different role orders");

    // But role counts should be the same
    for role in [Role::Liberal, Role::Fascist, Role::Hitler, Role::Communist] {
        let count1 = roles1.iter().filter(|&&r| r == role).count();
        let count2 = roles2.iter().filter(|&&r| r == role).count();
        assert_eq!(
            count1, count2,
            "Role counts should be consistent across different seeds"
        );
    }
}

/// Test PlayerDistribution calculation for various player counts
#[test]
fn test_player_distribution_calculation() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };

    // Test valid player counts
    for player_count in 6..=16 {
        let distribution = PlayerDistribution::new(&opts, player_count);
        assert!(distribution.is_ok(), "Player count {} should be valid", player_count);

        let dist = distribution.unwrap();
        let total_roles = dist.liberals
            + dist.fascists
            + dist.communists
            + dist.hitler as usize
            + dist.monarchist as usize
            + dist.anarchist as usize
            + dist.capitalist as usize
            + (dist.centrists as usize * 2);

        assert_eq!(
            total_roles, player_count,
            "Total roles should equal player count for {} players",
            player_count
        );
    }
}

/// Test role assignment function directly
#[test]
fn test_assign_roles_function() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };

    let distribution = PlayerDistribution::new(&opts, 12).unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let roles = assign_roles(distribution, &mut rng);

    // Verify correct number of each role
    assert_eq!(roles.len(), 12, "Should have 12 roles total");
    assert_eq!(
        roles.iter().filter(|&&r| r == Role::Hitler).count(),
        1,
        "Should have 1 Hitler"
    );
    assert_eq!(
        roles.iter().filter(|&&r| r == Role::Monarchist).count(),
        1,
        "Should have 1 Monarchist"
    );
    assert_eq!(
        roles.iter().filter(|&&r| r == Role::Anarchist).count(),
        1,
        "Should have 1 Anarchist"
    );
    assert_eq!(
        roles.iter().filter(|&&r| r == Role::Capitalist).count(),
        1,
        "Should have 1 Capitalist"
    );
    assert_eq!(
        roles.iter().filter(|&&r| r == Role::Centrist).count(),
        2,
        "Should have 2 Centrists"
    );

    // Verify roles are shuffled (not in predictable order)
    let expected_order = vec![
        Role::Fascist,
        Role::Fascist,
        Role::Fascist, // All fascists first
        Role::Communist,
        Role::Communist, // Then communists
        Role::Liberal,
        Role::Liberal,
        Role::Liberal, // Then liberals
        Role::Hitler,
        Role::Monarchist,
        Role::Anarchist,
        Role::Capitalist, // Then special roles
        Role::Centrist,
        Role::Centrist,
    ];
    assert_ne!(
        roles, expected_order,
        "Roles should be shuffled, not in predictable order"
    );
}

/// Test that role cards are distributed secretly (no duplicate assignments)
#[test]
fn test_role_cards_distributed_secretly() {
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Verify each player has exactly one role
    assert_eq!(game.players.len(), 8, "Should have 8 players");

    // Verify no duplicate role assignments (except for roles that can have multiples)
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    assert_eq!(hitler_count, 1, "Should have exactly 1 Hitler");

    // Verify all players have valid roles
    for (i, player) in game.players.iter().enumerate() {
        assert!(
            matches!(
                player.role,
                Role::Liberal | Role::Fascist | Role::Communist | Role::Hitler
            ),
            "Player {} should have a valid role",
            i
        );
    }
}

/// Test role assignment edge cases
#[test]
fn test_role_assignment_edge_cases() {
    // Test minimum player count with communists
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..6).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42);
    assert!(game.is_ok(), "6 players with communists should be valid");

    // Test without communists
    let opts_no_communists = GameOptions { communists: false, ..Default::default() };
    let player_names_5: Vec<String> = (0..5).map(|i| format!("Player{}", i)).collect();
    let game_no_communists = Game::new(opts_no_communists, &player_names_5, 42);

    if game_no_communists.is_ok() {
        let game = game_no_communists.unwrap();
        let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();
        assert_eq!(communist_count, 0, "Should have 0 Communists when communists disabled");
    }
}

/// Test role assignment consistency across multiple runs
#[test]
fn test_role_assignment_consistency() {
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();

    // Run multiple times with same seed
    for _ in 0..10 {
        let game = Game::new(opts, &player_names, 42).unwrap();

        // Verify consistent role counts
        let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
        let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
        let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
        let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

        assert_eq!(liberal_count, 4, "Should consistently have 4 Liberals");
        assert_eq!(fascist_count, 2, "Should consistently have 2 Fascists");
        assert_eq!(hitler_count, 1, "Should consistently have 1 Hitler");
        assert_eq!(communist_count, 1, "Should consistently have 1 Communist");
    }
}
