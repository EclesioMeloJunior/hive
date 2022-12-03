mod network;
mod node;
mod protocol;

use crate::network::{behaviour::HiveBehavior, transport::new_transport};
use libp2p::{identity, swarm::SwarmBuilder, Multiaddr, PeerId, Swarm};
use std::error::Error;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(local_key.public());
    println!("local peer id: {:?}", peer_id);

    let (transport, _) = new_transport(local_key);

    let behavior = {
        let hive_behaviour = HiveBehavior::new(peer_id);
        //let mdns_behavior = mdns::async_io::Behaviour::new(mdns::Config::default());
        //mdns::async_io::Behaviour::from(Some(hive_behaviour));

        hive_behaviour
    };

    let mut swarm = SwarmBuilder::with_async_std_executor(transport, behavior, peer_id).build();

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
