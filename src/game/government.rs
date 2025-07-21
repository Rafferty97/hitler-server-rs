use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub struct Government {
    pub president: usize,
    pub chancellor: usize,
}
