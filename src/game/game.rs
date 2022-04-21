use tokio::sync::broadcast;
use tracing::{span, Level, instrument, event};
use std::time::Instant;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::game::user::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub struct Game {
    started: Instant,
    users: Mutex<HashMap<Uuid, super::user::User>>,
    // the receiver we give to every user upon creation
    channel: (Sender<String>, Receiver<String>),

}

impl Game {
    pub fn new() -> Game {
        Game {
            started: Instant::now(),
            users: Mutex::new(HashMap::new()),
            channel: unbounded(),


        }
    }
}

#[derive(Debug)]
pub struct GameState {
    pub user_set: Mutex<HashMap<Uuid, User>>,
    pub tx: broadcast::Sender<String>,
    game: Arc<Mutex<Game>>,
}

impl GameState {
    #[instrument]
    pub fn new(game: Arc<Mutex<Game>>) -> GameState {
        GameState {
            user_set: Mutex::new(HashMap::new()),
            tx: broadcast::channel(100).0,
            game,
        }
    }

    #[instrument]
    pub async fn rolldice(&self) {
        event!(tracing::Level::INFO, "implement rolling dice: {:?}", self)
    }
}
