use uuid::Uuid;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::time::Instant;

#[derive(Debug)]
pub struct User {
    //wallet_address: VerifyingKey,
    pub uuid: Uuid,
    pub nick: String,
    hb: Instant,
}

impl User {
   pub fn new(uuid: Uuid) -> User {
        User {
            uuid,
            nick: "Anonymous".to_string(),
            hb: Instant::now(),
        }
    }

    pub async fn handle_socket() {

    }

}
