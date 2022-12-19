use serde::{Deserialize, Serialize};

use super::party::Party;

/// A game player.
#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub role: Role,
    pub alive: bool,
    pub not_hitler: bool,
    pub investigated: bool,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Liberal,
    Fascist,
    Hitler,
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::Liberal => "Liberal",
            Role::Fascist => "Fascist",
            Role::Hitler => "Hitler",
        }
        .to_string()
    }
}

impl Player {
    pub fn new(name: String, role: Role) -> Self {
        Self {
            name,
            role,
            alive: true,
            not_hitler: false,
            investigated: false,
        }
    }

    pub fn party(&self) -> Party {
        match self.role {
            Role::Liberal => Party::Liberal,
            Role::Fascist => Party::Fascist,
            Role::Hitler => Party::Fascist,
        }
    }
}

pub struct RoleAssigner {
    /// The indices of the fascist players; the first fascist is always hitler
    fascists: [usize; 4],
}

impl RoleAssigner {
    pub fn new(num_players: usize, rng: &mut impl rand::Rng) -> Self {
        let num_fascists = (num_players - 1) / 2;
        let mut fascists = [usize::MAX; 4];
        for i in 0..num_fascists {
            loop {
                let index = rng.gen_range(0..num_players);
                if fascists.iter().take(i).all(|i| *i != index) {
                    fascists[i] = index;
                    break;
                }
            }
        }
        Self { fascists }
    }

    pub fn get(&self, index: usize) -> Role {
        match self.fascists.iter().position(|i| *i == index) {
            Some(0) => Role::Hitler,
            Some(_) => Role::Fascist,
            None => Role::Liberal,
        }
    }
}
