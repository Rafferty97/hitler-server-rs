use serde::{Deserialize, Serialize};

/// Options for customising the game of Secret Hitler or Secret Hitler XL.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
pub struct GameOptions {
    /// Whether to include the communists in the game.
    pub communists: bool,
    /// Whether to include the monarchist.
    pub monarchist: bool,
    /// Whether to include the anarchist.
    pub anarchist: bool,
    /// Whether to include the capitalist.
    pub capitalist: bool,
    /// Whether to include the centrists.
    pub centrists: bool,
}

impl GameOptions {
    pub fn min_players(&self) -> usize {
        if self.communists {
            6
        } else {
            5
        }
    }

    pub fn max_players(&self) -> usize {
        if self.communists {
            16
        } else {
            10
        }
    }
}
