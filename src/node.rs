use crate::network::{behaviour::HiveBehavior, protocol::RequestVote};

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
    pub transport: Swarm<HiveBehavior>,
}

impl Node {
    pub fn new(swarm: Swarm<HiveBehavior>) -> Self {
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
                    Ok(ok_line) => {
                        let request_vote = RequestVote{
                            term: ok_line.parse::<u32>().unwrap(),
                            candidate_id: 1,
                            last_log_term: 1,
                            last_log_index: 1,
                        };

                        self.transport.behaviour_mut().publish(request_vote);
                    },
                    Err (err) => println!("cannot read line: {:?}", err)
                },

                event = self.transport.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {address:?}");
                    }

                    SwarmEvent::Behaviour(str) => {
                        println!(
                            "Received: '{:?}'", str)
                    }

                    // SwarmEvent::Behaviour(
                    //     OutEvent::Floodsub(
                    //         FloodsubEvent::Message(message))) =>
                    // {
                    //     println!(
                    //         "Received: '{:?}' form '{:?}'",
                    //         String::from_utf8_lossy(&message.data),
                    //         message.source,
                    //     )
                    // },

                    // SwarmEvent::Behaviour(
                    //     OutEvent::Mdns(
                    //         MdnsEvent::Discovered(list))) =>
                    // {
                    //     for (peer, _) in list {
                    //         self.transport
                    //             .behaviour_mut()
                    //             .floodsub
                    //             .add_node_to_partial_view(peer);

                    //         println!("Discovered node {:?}", peer);
                    //     }
                    // },

                    _ => {},
                }
            }
        }
    }
}
