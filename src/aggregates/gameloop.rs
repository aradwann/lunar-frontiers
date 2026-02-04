use uuid::Uuid;

use crate::commands::AdvanceGameloop;
use crate::events::{GameloopAdvanced, GameloopEvent};
use crate::models::BoxError;

#[derive(Debug, Clone)]
pub struct Gameloop {
    pub game_id: Uuid,
    pub tick: u64,
}

impl Gameloop {
    pub fn new(game_id: Uuid) -> Self {
        Self { game_id, tick: 0 }
    }

    /// Handle the AdvanceGameloop command
    pub fn handle_advance(&self, cmd: AdvanceGameloop) -> Result<GameloopEvent, BoxError> {
        Ok(GameloopEvent::Advanced(GameloopAdvanced {
            game_id: cmd.game_id,
            tick: cmd.tick,
        }))
    }

    /// Apply an event to update state
    pub fn apply(&mut self, event: &GameloopEvent) {
        match event {
            GameloopEvent::Advanced(evt) => {
                self.game_id = evt.game_id;
                self.tick = evt.tick;
            }
        }
    }

    /// Hydrate aggregate from event history
    pub fn hydrate(game_id: Uuid, events: Vec<GameloopEvent>) -> Self {
        let mut gameloop = Self::new(game_id);
        for event in events {
            gameloop.apply(&event);
        }
        gameloop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gameloop_advance() {
        let game_id = Uuid::now_v7();
        let gameloop = Gameloop::new(game_id);

        let cmd = AdvanceGameloop { game_id, tick: 1 };

        let event = gameloop.handle_advance(cmd).unwrap();

        match event {
            GameloopEvent::Advanced(evt) => {
                assert_eq!(evt.game_id, game_id);
                assert_eq!(evt.tick, 1);
            }
        }
    }

    #[test]
    fn test_gameloop_hydration() {
        let game_id = Uuid::now_v7();
        let events = vec![
            GameloopEvent::Advanced(GameloopAdvanced { game_id, tick: 1 }),
            GameloopEvent::Advanced(GameloopAdvanced { game_id, tick: 2 }),
            GameloopEvent::Advanced(GameloopAdvanced { game_id, tick: 3 }),
        ];

        let gameloop = Gameloop::hydrate(game_id, events);

        assert_eq!(gameloop.tick, 3);
        assert_eq!(gameloop.game_id, game_id);
    }
}
