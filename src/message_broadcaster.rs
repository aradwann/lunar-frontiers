use tokio::sync::broadcast;

use crate::events::{BuildingEvent, ConstructionSiteEvent, GameloopEvent};

pub type GameloopReceiver = broadcast::Receiver<GameloopEvent>;
pub type ConstructionReceiver = broadcast::Receiver<ConstructionSiteEvent>;
pub type BuildingReceiver = broadcast::Receiver<BuildingEvent>;

/// Message broadcaster for distributing events to subscribers
#[derive(Clone)]
pub struct MessageBroadcaster {
    gameloop_tx: broadcast::Sender<GameloopEvent>,
    construction_tx: broadcast::Sender<ConstructionSiteEvent>,
    building_tx: broadcast::Sender<BuildingEvent>,
}

impl MessageBroadcaster {
    pub fn new(capacity: usize) -> Self {
        let (gameloop_tx, _) = broadcast::channel(capacity);
        let (construction_tx, _) = broadcast::channel(capacity);
        let (building_tx, _) = broadcast::channel(capacity);

        Self {
            gameloop_tx,
            construction_tx,
            building_tx,
        }
    }

    pub fn subscribe_gameloop(&self) -> GameloopReceiver {
        self.gameloop_tx.subscribe()
    }

    pub fn subscribe_construction(&self) -> ConstructionReceiver {
        self.construction_tx.subscribe()
    }

    pub fn subscribe_building(&self) -> BuildingReceiver {
        self.building_tx.subscribe()
    }

    pub fn broadcast_gameloop(
        &self,
        event: GameloopEvent,
    ) -> Result<usize, broadcast::error::SendError<GameloopEvent>> {
        self.gameloop_tx.send(event)
    }

    pub fn broadcast_construction(
        &self,
        event: ConstructionSiteEvent,
    ) -> Result<usize, broadcast::error::SendError<ConstructionSiteEvent>> {
        self.construction_tx.send(event)
    }

    pub fn broadcast_building(
        &self,
        event: BuildingEvent,
    ) -> Result<usize, broadcast::error::SendError<BuildingEvent>> {
        self.building_tx.send(event)
    }
}
