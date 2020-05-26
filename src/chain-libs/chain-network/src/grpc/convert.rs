use super::proto;
use crate::data::{
    block::{self, Block, BlockEvent, BlockId, ChainPullRequest, Header},
    fragment::Fragment,
    gossip::{Gossip, Node},
    p2p::Peer,
};
use crate::error::{self, Error};
use tonic::{Code, Status};

use std::convert::TryFrom;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub(super) fn error_into_grpc(err: Error) -> Status {
    use error::Code::*;

    let code = match err.code() {
        Canceled => Code::Cancelled,
        Unknown => Code::Unknown,
        InvalidArgument => Code::InvalidArgument,
        NotFound => Code::NotFound,
        FailedPrecondition => Code::FailedPrecondition,
        Aborted => Code::Aborted,
        Unimplemented => Code::Unimplemented,
        Internal => Code::Internal,
        Unavailable => Code::Unavailable,
        // When a new case has to be added here, remember to
        // add the corresponding case in error_from_grpc below.
    };

    Status::new(code, err.to_string())
}

pub(super) fn error_from_grpc(e: Status) -> Error {
    use error::Code::*;

    let code = match e.code() {
        Code::Cancelled => Canceled,
        Code::Unknown => Unknown,
        Code::InvalidArgument => InvalidArgument,
        Code::NotFound => NotFound,
        Code::FailedPrecondition => FailedPrecondition,
        Code::Aborted => Aborted,
        Code::Unimplemented => Unimplemented,
        Code::Internal => Internal,
        Code::Unavailable => Unavailable,
        _ => Unknown,
    };

    Error::new(code, e)
}

impl From<Error> for Status {
    #[inline]
    fn from(e: Error) -> Self {
        error_into_grpc(e)
    }
}

impl From<Status> for Error {
    #[inline]
    fn from(status: Status) -> Self {
        error_from_grpc(status)
    }
}

pub trait FromProtobuf<R>: Sized {
    fn from_message(message: R) -> Result<Self, Error>;
}

pub trait IntoProtobuf {
    type Message;
    fn into_message(self) -> Self::Message;
}

pub(super) fn from_protobuf_repeated<P, T>(message: Vec<P>) -> Result<Box<[T]>, Error>
where
    T: FromProtobuf<P>,
{
    message.into_iter().map(T::from_message).collect()
}

pub(super) fn into_protobuf_repeated<I>(iter: I) -> Vec<<I::Item as IntoProtobuf>::Message>
where
    I: IntoIterator,
    I::Item: IntoProtobuf,
{
    iter.into_iter().map(|item| item.into_message()).collect()
}

pub(super) fn ids_into_repeated_bytes<I>(ids: I) -> Vec<Vec<u8>>
where
    I: IntoIterator,
    I::Item: AsRef<[u8]>,
{
    ids.into_iter().map(|id| id.as_ref().to_vec()).collect()
}

impl FromProtobuf<proto::Block> for Block {
    fn from_message(message: proto::Block) -> Result<Self, Error> {
        Ok(Block::from_bytes(message.content))
    }
}

impl IntoProtobuf for Block {
    type Message = proto::Block;

    fn into_message(self) -> proto::Block {
        proto::Block {
            content: self.into(),
        }
    }
}

impl FromProtobuf<proto::Header> for Header {
    fn from_message(message: proto::Header) -> Result<Self, Error> {
        Ok(Header::from_bytes(message.content))
    }
}

impl IntoProtobuf for Header {
    type Message = proto::Header;

    fn into_message(self) -> proto::Header {
        proto::Header {
            content: self.into(),
        }
    }
}

impl FromProtobuf<proto::Fragment> for Fragment {
    fn from_message(message: proto::Fragment) -> Result<Self, Error> {
        Ok(Fragment::from_bytes(message.content))
    }
}

impl IntoProtobuf for Fragment {
    type Message = proto::Fragment;

    fn into_message(self) -> proto::Fragment {
        proto::Fragment {
            content: self.into(),
        }
    }
}

impl FromProtobuf<proto::Gossip> for Gossip {
    fn from_message(message: proto::Gossip) -> Result<Self, Error> {
        let gossip = Gossip {
            nodes: message
                .nodes
                .into_iter()
                .map(Node::from_bytes)
                .collect::<Vec<_>>()
                .into(),
        };
        Ok(gossip)
    }
}

impl IntoProtobuf for Gossip {
    type Message = proto::Gossip;

    fn into_message(self) -> proto::Gossip {
        proto::Gossip {
            nodes: self
                .nodes
                .into_vec()
                .into_iter()
                .map(|node| node.into_bytes())
                .collect(),
        }
    }
}

impl FromProtobuf<proto::Peer> for Peer {
    fn from_message(message: proto::Peer) -> Result<Self, Error> {
        use proto::peer;

        match message.peer {
            Some(peer::Peer::V4(pv4)) => {
                let port = pv4.port as u16;
                let segments = pv4.ip.to_be_bytes();
                let ipv4 = Ipv4Addr::new(segments[0], segments[1], segments[2], segments[3]);
                let addr = SocketAddr::new(IpAddr::V4(ipv4), port);
                Ok(addr.into())
            }
            Some(peer::Peer::V6(pv6)) => {
                let port = pv6.port as u16;
                let ipv6 = unserialize_ipv6(pv6.ip_high, pv6.ip_low);
                let addr = SocketAddr::new(IpAddr::V6(ipv6), port);
                Ok(addr.into())
            }
            None => Err(Error::new(
                error::Code::InvalidArgument,
                "invalid Peer payload, one of the fields is required",
            )),
        }
    }
}

fn unserialize_ipv6(high: u64, low: u64) -> Ipv6Addr {
    let h = high.to_be_bytes();
    let l = low.to_be_bytes();
    fn from_be_bytes(h: u8, l: u8) -> u16 {
        u16::from_be_bytes([h, l])
    }
    let segments: [u16; 8] = [
        from_be_bytes(h[0], h[1]),
        from_be_bytes(h[2], h[3]),
        from_be_bytes(h[4], h[5]),
        from_be_bytes(h[6], h[7]),
        from_be_bytes(l[0], l[1]),
        from_be_bytes(l[2], l[3]),
        from_be_bytes(l[4], l[5]),
        from_be_bytes(l[6], l[7]),
    ];
    std::net::Ipv6Addr::new(
        segments[0],
        segments[1],
        segments[2],
        segments[3],
        segments[4],
        segments[5],
        segments[6],
        segments[7],
    )
}

impl IntoProtobuf for Peer {
    type Message = proto::Peer;

    fn into_message(self) -> proto::Peer {
        let peer = match self.addr() {
            SocketAddr::V4(v4addr) => {
                let port: u32 = v4addr.port().into();
                let ip = u32::from_be_bytes(v4addr.ip().octets());
                proto::peer::Peer::V4(proto::PeerV4 { ip, port })
            }
            SocketAddr::V6(v6addr) => {
                let port: u32 = v6addr.port().into();
                let (ip_high, ip_low) = serialize_ipv6(v6addr.ip());
                proto::peer::Peer::V6(proto::PeerV6 {
                    port,
                    ip_high,
                    ip_low,
                })
            }
        };
        proto::Peer { peer: Some(peer) }
    }
}

fn serialize_ipv6(ip: &Ipv6Addr) -> (u64, u64) {
    let segs = ip.segments();
    let mut out = [0u64; 2];
    for i in 0..2 {
        let mut v = [0u8; 8];
        for j in 0..4 {
            let b: [u8; 2] = segs[i * 4 + j].to_be_bytes();
            v[j * 2] = b[0];
            v[j * 2 + 1] = b[1];
        }
        out[i] = u64::from_be_bytes(v)
    }
    (out[0], out[1])
}

impl FromProtobuf<proto::BlockEvent> for BlockEvent {
    fn from_message(msg: proto::BlockEvent) -> Result<Self, Error> {
        use proto::block_event::Item::*;

        match msg.item {
            Some(Announce(header)) => {
                let header = Header::from_message(header)?;
                Ok(BlockEvent::Announce(header))
            }
            Some(Solicit(block_ids)) => {
                let block_ids = block::try_ids_from_iter(block_ids.ids)?;
                Ok(BlockEvent::Solicit(block_ids))
            }
            Some(Missing(pull_req)) => {
                let from = block::try_ids_from_iter(pull_req.from)?;
                let to = BlockId::try_from(&pull_req.to[..])?;
                Ok(BlockEvent::Missing(ChainPullRequest { from, to }))
            }
            None => Err(Error::new(
                error::Code::InvalidArgument,
                "one of the BlockEvent variants must be present",
            )),
        }
    }
}

impl IntoProtobuf for BlockEvent {
    type Message = proto::BlockEvent;

    fn into_message(self) -> proto::BlockEvent {
        use proto::block_event::Item;
        let item = match self {
            BlockEvent::Announce(header) => Item::Announce(header.into_message()),
            BlockEvent::Solicit(block_ids) => {
                let block_ids = proto::BlockIds {
                    ids: ids_into_repeated_bytes(block_ids.iter()),
                };
                Item::Solicit(block_ids)
            }
            BlockEvent::Missing(ChainPullRequest { from, to }) => {
                let request = proto::PullHeadersRequest {
                    from: ids_into_repeated_bytes(from.iter()),
                    to: to.as_bytes().into(),
                };
                Item::Missing(request)
            }
        };
        proto::BlockEvent { item: Some(item) }
    }
}
