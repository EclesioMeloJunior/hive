mod network;

use std::error::Error;

use crate::protocol::{HiveBehaviour, HiveBehaviourEvent};
use futures::{
    prelude::{stream::StreamExt, *},
    select,
};
use libp2p::{
    gossipsub::{GossipsubEvent, IdentTopic},
    identity::{self, Keypair},
    mdns,
    multihash::Multihash,
    swarm::{DialError, SwarmEvent},
    Multiaddr, Swarm,
};

use network::NetworkLayer;

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

#[derive(Debug)]
pub enum DialWithPeerError {
    NetworkLayerFailed(DialError),
    UnexpectedPeerId(String),
}

impl std::fmt::Display for DialWithPeerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NetworkLayerFailed(dial_error) => write!(f, "{}", dial_error),
            Self::UnexpectedPeerId(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for DialWithPeerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NetworkLayerFailed(dial_error) => Some(dial_error),
            Self::UnexpectedPeerId(s) => None,
        }
    }
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
        let mut stdin = async_std::io::BufReader::new(async_std::io::stdin())
            .lines()
            .fuse();

        loop {
            select! {
                line = stdin.select_next_some() => {
                    let topic = IdentTopic::new("request_topic");
                    let publish = self.network.publish(topic, line.expect("failed to read input line").as_bytes());

                    if let Err(err) = publish {
                        println!("Publish error: {:?}", err);
                    }
                },


                event = self.network.next_event() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {address:?}");
                    },

                    SwarmEvent::Behaviour(HiveBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discovered a new peer: {}", peer_id);
                            self.network.add_explicit_peer(&peer_id);
                        }
                    },

                    SwarmEvent::Behaviour(HiveBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discovered a new peer: {}", peer_id);
                            self.network.remove_explicit_peer(&peer_id);
                        }
                    },

                    SwarmEvent::Behaviour(HiveBehaviourEvent::Gossipsub(GossipsubEvent::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => {
                        println!(
                            "Got message: '{}' with id: {id} from peer: {peer_id}",
                            String::from_utf8_lossy(&message.data),
                        );
                    },
                    _ => {},
                }
            }
        }
    }
}
