use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AdvanceGameloop {
    pub game_id: Uuid,
    pub tick: u64,
}
