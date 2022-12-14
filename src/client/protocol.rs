use libp2p::{gossipsub::Gossipsub, mdns, swarm::NetworkBehaviour};
use parity_scale_codec_derive::{Decode, Encode};

#[derive(NetworkBehaviour)]
pub struct HiveBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: mdns::async_io::Behaviour,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct AppendEntries {
    pub term: u32,
    pub leader_id: u32,
    pub leader_commit: u32,
    pub entries: Vec<u8>,
    pub previous_log_term: u32,
    pub previous_log_index: u32,
}
