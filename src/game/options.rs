use std::ops::RangeInclusive;

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
