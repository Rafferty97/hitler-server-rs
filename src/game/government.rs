use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Government {
    pub president: usize,
    pub chancellor: usize,
}
