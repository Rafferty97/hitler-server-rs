use serde::{Deserialize, Serialize};

/// The two political parties of the game.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Party {
    Liberal,
    Fascist,
    Communist,
}

impl ToString for Party {
    fn to_string(&self) -> String {
        match self {
            Party::Liberal => "Liberal",
            Party::Fascist => "Fascist",
            Party::Communist => "Communist",
        }
        .to_string()
    }
}
