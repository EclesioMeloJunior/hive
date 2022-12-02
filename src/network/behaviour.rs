use cuckoofilter::{CuckooError, CuckooFilter};
use libp2p::{
    swarm::{
        derive_prelude::{ConnectionClosed, ConnectionEstablished, FromSwarm},
        dial_opts::DialOpts,
        NetworkBehaviour, NetworkBehaviourAction, NotifyHandler, OneShotHandler, PollParameters,
    },
    PeerId,
};
use std::{
    collections::{hash_map::DefaultHasher, HashSet, VecDeque},
    task::{Context, Poll},
};

use super::protocol::{self, HiveRequestVoteProtocol, RequestVote};

pub struct HiveBehavior {
    events: VecDeque<
        NetworkBehaviourAction<
            HiveEvent,
            OneShotHandler<HiveRequestVoteProtocol, RequestVote, InnerMessage>,
        >,
    >,

    local_peer_id: PeerId,

    // List of peers to send message to
    connected_peers: HashSet<PeerId>,

    // keep track of received messages to avoid sending a message twice
    // if we receive it twice from the network
    received_messages: CuckooFilter<DefaultHasher>,
}

impl HiveBehavior {
    pub fn new(local_peer_id: PeerId) -> Self {
        HiveBehavior {
            events: VecDeque::new(),
            local_peer_id: local_peer_id,
            connected_peers: HashSet::new(),
            received_messages: CuckooFilter::new(),
        }
    }

    #[inline]
    pub fn add_node_to_partial_view(&mut self, peer_id: PeerId) {
        if self.connected_peers.insert(peer_id) {
            let handler = self.new_handler();
            self.events.push_back(NetworkBehaviourAction::Dial {
                opts: DialOpts::peer_id(peer_id).build(),
                handler,
            })
        }
    }

    fn on_connection_established(&mut self, conn_established: ConnectionEstablished) {
        println!("trying to establish a conn: {:?}", conn_established.peer_id);
        if conn_established.other_established > 0 {
            return;
        }

        println!("connection established: {:?}", conn_established.peer_id);
        self.connected_peers.insert(conn_established.peer_id);
    }

    fn on_connection_closed(
        &mut self,
        conn_closed: ConnectionClosed<<Self as NetworkBehaviour>::ConnectionHandler>,
    ) {
        if conn_closed.remaining_established > 0 {
            return;
        }

        let was_in = self.connected_peers.remove(&conn_closed.peer_id);
        if was_in {
            println!("connection closed: {:?}", conn_closed.peer_id);
        }

        let handler = self.new_handler();
        self.events.push_back(NetworkBehaviourAction::Dial {
            opts: DialOpts::peer_id(conn_closed.peer_id).build(),
            handler: handler,
        });
    }

    pub fn publish(&mut self, request_vote: RequestVote) {
        for conn_peer in self.connected_peers.iter() {
            self.events
                .push_back(NetworkBehaviourAction::NotifyHandler {
                    peer_id: *conn_peer,
                    handler: NotifyHandler::Any,
                    event: request_vote,
                })
        }
    }
}

impl NetworkBehaviour for HiveBehavior {
    type ConnectionHandler = OneShotHandler<HiveRequestVoteProtocol, RequestVote, InnerMessage>;
    type OutEvent = HiveEvent;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        Default::default()
    }

    fn poll(
        &mut self,
        _: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        if let Some(event) = self.events.pop_front() {
            return Poll::Ready(event);
        }

        Poll::Pending
    }

    fn on_connection_handler_event(
        &mut self,
        propagation_source: PeerId,
        _connection_id: libp2p::swarm::derive_prelude::ConnectionId,
        event: <<Self::ConnectionHandler as libp2p::swarm::IntoConnectionHandler>::Handler as
            libp2p::swarm::ConnectionHandler>::OutEvent,
    ) {
        let event = match event {
            InnerMessage::Rx(event) => event,
            InnerMessage::Sent => return,
        };

        if self.received_messages.contains(&event) {
            return;
        }

        match self.received_messages.add(&event) {
            Ok(_) => return,
            Err(CuckooError::NotEnoughSpace) => {
                println!(
                    "message included but another random message
                was removed"
                )
            }
        };

        println!("on connection handler event: {:?}", event);
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        match event {
            FromSwarm::ConnectionEstablished(conn) => {
                self.on_connection_established(conn);
            }
            FromSwarm::ConnectionClosed(conn) => {
                self.on_connection_closed(conn);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum HiveEvent {
    RequestVote(protocol::RequestVote),
}

#[derive(Debug, Clone)]
pub enum InnerMessage {
    Rx(protocol::RequestVote),
    Sent,
}

impl From<protocol::RequestVote> for InnerMessage {
    #[inline]
    fn from(value: protocol::RequestVote) -> Self {
        InnerMessage::Rx(value)
    }
}

impl From<()> for InnerMessage {
    #[inline]
    fn from(value: ()) -> Self {
        InnerMessage::Sent
    }
}
