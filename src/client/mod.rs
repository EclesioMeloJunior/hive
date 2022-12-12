mod network;
mod protocol;

use std::error::Error;

use libp2p::identity::{self, Keypair};

use network::{DialWithPeerError, NetworkLayer};

#[derive(Debug)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

pub struct Node {
    pub role: Role,
    pub local_key: Keypair,
    pub network: NetworkLayer,
}

impl Node {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        match NetworkLayer::new_with_keypair(local_key.clone(), vec!["request_topic"]) {
            Ok(mut net_layer) => {
                net_layer.listen_on("/ip4/0.0.0.0/tcp/0")?;

                Ok(Node {
                    local_key,
                    role: Role::Follower,
                    network: net_layer,
                })
            }

            Err(err) => Err(err),
        }
    }

    pub fn connect_with(&mut self, peer_to_dial: &str) -> Result<(), DialWithPeerError> {
        match peer_to_dial.parse() {
            Ok(to_dial_addr) => match self.network.connect_with(to_dial_addr) {
                Ok(_) => Ok(()),
                Err(err) => Err(DialWithPeerError::NetworkLayerFailed(err)),
            },
            Err(_) => Err(DialWithPeerError::UnexpectedPeerId(
                peer_to_dial.to_string(),
            )),
        }
    }

    pub async fn start(&mut self) {
        loop {
            self.network.next_event().await;
        }
    }
}
