use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameloopAdvanced {
    pub game_id: Uuid,
    pub tick: u64,
}
// Aggregate event enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameloopEvent {
    Advanced(GameloopAdvanced),
}
