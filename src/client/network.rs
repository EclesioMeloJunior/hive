use futures::{prelude::*, select};
use libp2p::{
    gossipsub::{
        error::PublishError, Gossipsub, GossipsubConfigBuilder, GossipsubEvent, GossipsubMessage,
        IdentTopic, MessageAuthenticity, MessageId, ValidationMode,
    },
    identity::Keypair,
    mdns,
    swarm::{DialError, SwarmBuilder, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use std::{
    collections::hash_map::DefaultHasher, error::Error, hash::Hash, hash::Hasher, time::Duration,
};

use crate::{
    client::protocol::{HiveBehaviour, HiveBehaviourEvent},
    network::transport::new_transport,
};

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

pub struct NetworkLayer {
    peer_id: PeerId,
    topics: Vec<IdentTopic>,
    transport: Swarm<HiveBehaviour>,
}

impl NetworkLayer {
    pub fn new_with_keypair(keypair: Keypair, topics: Vec<&str>) -> Result<Self, Box<dyn Error>> {
        let peer_id = PeerId::from(keypair.public());
        let (transport, _) = new_transport(keypair.clone());

        let message_fn_id = |message: &GossipsubMessage| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            MessageId::from(hasher.finish().to_string())
        };

        let gossipsub_config = GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_fn_id)
            .build()
            .expect("while create gossipsub config");

        let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(keypair), gossipsub_config)
            .expect("while createing gossipsub new");

        let mut supported_topics: Vec<IdentTopic> = vec![];
        for topic in topics {
            let current_topic = IdentTopic::new(topic);
            gossipsub.subscribe(&current_topic)?;
            supported_topics.push(current_topic);
        }

        let swarm = {
            let mdns = mdns::async_io::Behaviour::new(mdns::Config::default())?;
            let behaviour = HiveBehaviour { gossipsub, mdns };
            SwarmBuilder::with_async_std_executor(transport, behaviour, peer_id).build()
        };

        Ok(NetworkLayer {
            peer_id: peer_id,
            transport: swarm,
            topics: supported_topics,
        })
    }

    pub fn listen_on(&mut self, addr: &str) -> Result<(), Box<dyn Error>> {
        Swarm::listen_on(&mut self.transport, addr.parse()?)?;
        Ok(())
    }

    pub fn connect_with(&mut self, peer_to_dial: Multiaddr) -> Result<(), DialError> {
        self.transport.dial(peer_to_dial)
    }
}

impl NetworkLayer {
    pub fn publish<'a, T>(
        &mut self,
        topic: IdentTopic,
        message: T,
    ) -> Result<MessageId, PublishError>
    where
        T: Into<Vec<u8>>,
    {
        self.transport
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), message)
    }

    pub fn add_explicit_peer(&mut self, peer_id: &PeerId) {
        self.transport
            .behaviour_mut()
            .gossipsub
            .add_explicit_peer(peer_id)
    }

    pub fn remove_explicit_peer(&mut self, peer_id: &PeerId) {
        self.transport
            .behaviour_mut()
            .gossipsub
            .remove_explicit_peer(&peer_id)
    }
}

impl NetworkLayer {
    pub async fn next_event(&mut self) {
        select! {
            event = self.transport.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {address:?}");
                },

                SwarmEvent::Behaviour(HiveBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {}", peer_id);
                        self.transport.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },

                SwarmEvent::Behaviour(HiveBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {}", peer_id);
                        self.transport.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
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
