use parity_scale_codec_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct AppendEntries {
    pub term: u32,
    pub leader_id: u32,
    pub leader_commit: u32,
    pub entries: Vec<u8>,
    pub previous_log_term: u32,
    pub previous_log_index: u32,
}
