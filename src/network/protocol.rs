use futures::{
    io::{AsyncRead, AsyncWrite},
    AsyncWriteExt, Future,
};
use libp2p::core::{upgrade, InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use parity_scale_codec::{Decode, Encode};
use parity_scale_codec_derive::{Decode, Encode};
use std::{io, iter, pin::Pin};

/// Implementation of the ConnectionUpgrade for my protocol

#[derive(Debug, Clone, Default)]
pub struct HiveRequestVoteProtocol {}

impl HiveRequestVoteProtocol {
    pub fn new() -> Self {
        HiveRequestVoteProtocol {}
    }
}

impl UpgradeInfo for HiveRequestVoteProtocol {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/hive/request_vote/1.0.0")
    }
}

impl<TSocket> InboundUpgrade<TSocket> for HiveRequestVoteProtocol
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = RequestVote;
    type Error = ProtocolError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>>;

    fn upgrade_inbound(self, mut socket: TSocket, _: Self::Info) -> Self::Future {
        Box::pin(async move {
            let message = upgrade::read_length_prefixed(&mut socket, 2048).await;

            let message = match message {
                // TODO: log this error
                Err(_) => return Err(ProtocolError::ReadInboundStreamError),
                Ok(message) => message,
            };

            let request_vote = match RequestVote::decode(&mut message.as_slice()) {
                // TODO: log this error
                Err(_) => return Err(ProtocolError::DecodeError),
                Ok(request_vote) => request_vote,
            };

            Ok(request_vote)
        })
    }
}

#[derive(Debug, Clone)]
pub enum ProtocolError {
    ReadInboundStreamError,
    DecodeError,
}

#[derive(Debug, Copy, Clone, PartialEq, Encode, Decode, Hash)]
pub struct RequestVote {
    pub term: u32,
    pub candidate_id: u32,
    pub last_log_term: u32,
    pub last_log_index: u32,
}

impl UpgradeInfo for RequestVote {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/hive/request_vote/1.0.0")
    }
}

impl<TSocket> OutboundUpgrade<TSocket> for RequestVote
where
    TSocket: AsyncWrite + AsyncRead + Send + Unpin + 'static,
{
    type Output = ();
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>>;

    fn upgrade_outbound(self, mut socket: TSocket, _: Self::Info) -> Self::Future {
        Box::pin(async move {
            let bytes = self.encode();
            upgrade::write_length_prefixed(&mut socket, bytes).await?;
            socket.close().await?;

            Ok(())
        })
    }
}
