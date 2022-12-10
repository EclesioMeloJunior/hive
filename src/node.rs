use crate::protocol::HiveBehaviour;
use futures::{
    prelude::{stream::StreamExt, *},
    select,
};
use libp2p::{swarm::SwarmEvent, Swarm};
#[derive(Debug)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

pub struct Node {
    pub role: Role,
    pub transport: Swarm<HiveBehaviour>,
}

impl Node {
    pub fn new(swarm: Swarm<HiveBehaviour>) -> Self {
        Node {
            role: Role::Follower,
            transport: swarm,
        }
    }

    pub async fn start(&mut self) {
        let mut stdin = async_std::io::BufReader::new(async_std::io::stdin())
            .lines()
            .fuse();

        loop {
            select! {
                line = stdin.select_next_some() => match line {
                    self.transport
                        .behaviour_mut().gossipsub
                        .publish(topic.clone(), line.expect("Stdin not to close").as_bytes());
                },

                event = self.transport.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {address:?}");
                    }

                    SwarmEvent::Behaviour(HiveEvent::RequestVote(request_vote)) => {
                            println!("Received: '{:?}'", request_vote)
                    }
                    _ => {},
                }
            }
        }
    }
}
