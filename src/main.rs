mod client;
mod network;
mod protocol;

use crate::client::Node;
use std::error::Error;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut node = Node::new()?;

    // connect to another peer
    if let Some(to_dial) = std::env::args().nth(1) {
        node.connect_with(to_dial.as_str())?;
        println!("Dialed {:?}", to_dial);
    }

    node.start().await;

    Ok(())
}
