mod network;
mod node;
mod protocol;

use crate::network::transport::new_transport;
use crate::protocol::HiveBehaviour;
use libp2p::{
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubMessage, IdentTopic, MessageAuthenticity,
        MessageId, ValidationMode,
    },
    identity, mdns,
    swarm::SwarmBuilder,
    Multiaddr, PeerId, Swarm,
};
use std::{collections::hash_map::DefaultHasher, error::Error, hash::Hasher};
use std::{hash::Hash, time::Duration};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(local_key.public());
    println!("local peer id: {:?}", peer_id);

    let (transport, _) = new_transport(local_key.clone());

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

    let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
        .expect("while createing gossipsub new");

    let request_vote_topic: IdentTopic = IdentTopic::new("request_vote");
    gossipsub.subscribe(&request_vote_topic);

    let mut swarm = {
        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default())?;
        let behaviour = HiveBehaviour { gossipsub, mdns };
        SwarmBuilder::with_async_std_executor(transport, behaviour, peer_id).build()
    };

    let addr = "/ip4/0.0.0.0/tcp/0";
    if let Err(err) = Swarm::listen_on(&mut swarm, addr.parse()?) {
        println!("Cannot listen on {} because: {:?}", addr, err)
    }

    // connect to another peer
    if let Some(to_dial) = std::env::args().nth(1) {
        let to_dial_addr: Multiaddr = to_dial.parse()?;
        swarm.dial(to_dial_addr)?;
        println!("Dialed {:?}", to_dial);
    }

    let mut node = node::Node::new(swarm);
    node.start().await;

    Ok(())
}
