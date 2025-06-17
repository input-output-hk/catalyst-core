use super::{buffer_sizes, convert::Decode, p2p::comm::GetNodeAddress, GlobalStateR};
use crate::{
    blockcfg::Fragment,
    intercom::{self, BlockMsg, TopologyMsg, TransactionMsg},
    network::retrieve_local_ip,
    settings::start::network::Configuration,
    topology::{Gossip, NodeId},
    utils::async_msg::{self, MessageBox},
};
use chain_network::{
    data as net_data,
    error::{Code, Error},
};
use futures::{future::BoxFuture, prelude::*, ready};
use jormungandr_lib::interfaces::FragmentOrigin;
use std::{
    error::Error as _,
    mem,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tracing_futures::Instrument;

/// Conditionally filter gossip from non-public IP addresses
fn filter_gossip_node(node: &Gossip, config: &Configuration) -> bool {
    if config.allow_private_addresses {
        node.has_valid_address()
    } else {
        !node.is_global()
    }
}

fn handle_mbox_error(err: async_msg::SendError) -> Error {
    tracing::error!(
        reason = %err,
        "failed to send block announcement to the block task"
    );
    Error::new(Code::Internal, err)
}

pub async fn process_block_announcements<S>(
    stream: S,
    mbox: MessageBox<BlockMsg>,
    node_id: NodeId,
    global_state: GlobalStateR,
) where
    S: TryStream<Ok = net_data::Header, Error = Error>,
{
    let sink = BlockAnnouncementProcessor::new(mbox, node_id, global_state);
    stream
        .into_stream()
        .forward(sink)
        .await
        .unwrap_or_else(|e| {
            tracing::debug!(error = ?e, "processing of inbound subscription stream failed");
        });
}

pub async fn process_gossip<S>(
    stream: S,
    mbox: MessageBox<TopologyMsg>,
    node_id: NodeId,
    global_state: GlobalStateR,
) where
    S: TryStream<Ok = net_data::Gossip, Error = Error>,
{
    let processor = GossipProcessor::new(mbox, node_id, global_state, Direction::Server);
    stream
        .into_stream()
        .forward(processor)
        .await
        .unwrap_or_else(|e| {
            tracing::debug!(
                error = ?e,
                "processing of inbound gossip failed"
            );
        });
}

pub async fn process_fragments<S>(
    stream: S,
    mbox: MessageBox<TransactionMsg>,
    node_id: NodeId,
    global_state: GlobalStateR,
) where
    S: TryStream<Ok = net_data::Fragment, Error = Error>,
{
    let sink = FragmentProcessor::new(mbox, node_id, global_state);
    stream
        .into_stream()
        .forward(sink)
        .await
        .unwrap_or_else(|e| {
            tracing::debug!(error = ?e, "processing of inbound subscription stream failed");
        });
}

#[must_use = "sinks do nothing unless polled"]
pub struct BlockAnnouncementProcessor {
    mbox: MessageBox<BlockMsg>,
    node_id: NodeId,
    global_state: GlobalStateR,
    pending_processing: PendingProcessing,
}

impl BlockAnnouncementProcessor {
    pub(super) fn new(
        mbox: MessageBox<BlockMsg>,
        node_id: NodeId,
        global_state: GlobalStateR,
    ) -> Self {
        BlockAnnouncementProcessor {
            mbox,
            node_id,
            global_state,
            pending_processing: PendingProcessing::default(),
        }
    }

    pub fn message_box(&self) -> MessageBox<BlockMsg> {
        self.mbox.clone()
    }

    fn refresh_stat(&mut self) {
        let state = self.global_state.clone();
        let node_id = self.node_id;
        let fut = async move {
            let refreshed = state.peers.refresh_peer_on_block(&node_id).await;
            if !refreshed {
                tracing::debug!("received block from node that is not in the peer map");
            }
        }
        .in_current_span();
        // It's OK to overwrite a pending future because only the latest
        // timestamp matters.
        self.pending_processing.start(fut);
    }

    fn poll_flush_mbox(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.mbox)
            .poll_flush(cx)
            .map_err(handle_mbox_error)
    }
}

pub enum Direction {
    Server,
    Client,
}

pub struct GossipProcessor {
    mbox: MessageBox<TopologyMsg>,
    node_id: NodeId,
    global_state: GlobalStateR,
    pending_processing: PendingProcessing,
    // To keep a healthy pool of p2p peers, we need to keep track of nodes we were able
    // to connect to successfully.
    // However, a server may need to accomodate peers which are not publicy reachable
    // (e.g. private nodes, full wallets, ...) and embedding this process in the handshake
    // procedure is not the best idea.
    // Instead, a peer is "promoted" (i.e. marked as successfully connected in poldercast terminology)
    // after the first gossip is received, which signals interest in participating in the dissemination
    // overlay.
    peer_promoted: bool,
}

impl GossipProcessor {
    pub(super) fn new(
        mbox: MessageBox<TopologyMsg>,
        node_id: NodeId,
        global_state: GlobalStateR,
        direction: Direction,
    ) -> Self {
        GossipProcessor {
            mbox,
            node_id,
            global_state,
            pending_processing: Default::default(),
            // client will handle promotion after handshake since they are connecting to a public
            // node by construction
            peer_promoted: matches!(direction, Direction::Client),
        }
    }
}

impl Sink<net_data::Header> for BlockAnnouncementProcessor {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self.pending_processing.poll_complete(cx) {
            Poll::Pending => {
                ready!(self.as_mut().poll_flush_mbox(cx))?;
                Poll::Pending
            }
            Poll::Ready(()) => self.mbox.poll_ready(cx).map_err(handle_mbox_error),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, raw_header: net_data::Header) -> Result<(), Error> {
        let header = raw_header.decode().inspect_err(|e| {
            tracing::info!(
                reason = %e.source().unwrap(),
                "failed to decode incoming block announcement header"
            );
        })?;

        tracing::debug!(hash = %header.hash(), "received block announcement");

        let node_id = self.node_id;
        self.mbox
            .start_send(BlockMsg::AnnouncedBlock(Box::new(header), node_id))
            .map_err(handle_mbox_error)?;

        self.refresh_stat();

        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self.pending_processing.poll_complete(cx) {
            Poll::Pending => {
                ready!(self.as_mut().poll_flush_mbox(cx))?;
                Poll::Pending
            }
            Poll::Ready(()) => self.as_mut().poll_flush_mbox(cx),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self.pending_processing.poll_complete(cx) {
            Poll::Pending => {
                ready!(self.as_mut().poll_flush_mbox(cx))?;
                Poll::Pending
            }
            Poll::Ready(()) => Pin::new(&mut self.mbox).poll_close(cx).map_err(|e| {
                tracing::warn!(
                    reason = %e,
                    "failed to close communication channel to the block task"
                );
                Error::new(Code::Internal, e)
            }),
        }
    }
}

/// Future returned by trust-dns-resolver reverse lookup.
type ReverseDnsLookupFuture = Pin<
    Box<
        dyn Future<
                Output = Result<
                    trust_dns_resolver::lookup::ReverseLookup,
                    trust_dns_resolver::error::ResolveError,
                >,
            > + Send,
    >,
>;

/// Possible states for [FragmentProcessor]::poll_send_fragments.
enum FragmentProcessorSendFragmentsState {
    /// Wait for the message box to have capacity
    /// to send at least one fragment.
    WaitingMessageBox,
    /// Fetch the address of the inbound peer from which the [FragmentProcessor]
    /// is receiving fragments from.
    GetIngressAddress { fut: GetNodeAddress },
    /// Executes a reverse DNS lookup query on the inbound peer address.
    ReverseLookup {
        ingress_addr: SocketAddr,
        fut: ReverseDnsLookupFuture,
    },
    /// Checks the inbound peer address and (if resolved to any) resolved hostnames
    /// against the configured whitelist and sends fragments to the fragments task.
    SendFragments {
        ingress_addr: SocketAddr,
        resolved_hostnames: Vec<String>,
    },
}

impl std::fmt::Display for FragmentProcessorSendFragmentsState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_name = match self {
            FragmentProcessorSendFragmentsState::WaitingMessageBox => "WaitingMessageBox",
            FragmentProcessorSendFragmentsState::GetIngressAddress { .. } => "GetIngressAddress",
            FragmentProcessorSendFragmentsState::ReverseLookup { .. } => "ReverseLookup",
            FragmentProcessorSendFragmentsState::SendFragments { .. } => "SendFragments",
        };

        write!(f, "{}", state_name)
    }
}

/// Ingests inbound subscription stream, the node_id refers to the inbound peer
#[must_use = "sinks do nothing unless polled"]
pub struct FragmentProcessor {
    mbox: MessageBox<TransactionMsg>,
    node_id: NodeId,
    global_state: GlobalStateR,
    buffered_fragments: Vec<Fragment>,
    pending_processing: PendingProcessing,
    send_fragments_state: FragmentProcessorSendFragmentsState,
}

impl FragmentProcessor {
    pub(super) fn new(
        mbox: MessageBox<TransactionMsg>,
        node_id: NodeId,
        global_state: GlobalStateR,
    ) -> Self {
        FragmentProcessor {
            mbox,
            node_id,
            global_state,
            buffered_fragments: Vec::with_capacity(buffer_sizes::inbound::FRAGMENTS),
            pending_processing: PendingProcessing::default(),
            send_fragments_state: FragmentProcessorSendFragmentsState::WaitingMessageBox,
        }
    }

    /// Signals interaction with peer has occurred, this context relates to exchanging fragments.
    fn refresh_stat(&mut self) {
        let state = self.global_state.clone();
        let node_id = self.node_id;
        let fut = async move {
            let refreshed = state.peers.refresh_peer_on_fragment(&node_id).await;
            if !refreshed {
                tracing::debug!("received fragment from node that is not in the peer map",);
            }
        }
        .in_current_span();
        self.pending_processing.start(fut);
    }

    /// Sends fragments to the fragments task.
    fn poll_send_fragments(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let st = std::mem::replace(
            &mut self.send_fragments_state,
            FragmentProcessorSendFragmentsState::WaitingMessageBox,
        );
        tracing::debug!(current_state = %st, "starting poll_send_fragments");

        let (next_st, ret) = match st {
            FragmentProcessorSendFragmentsState::WaitingMessageBox => {
                match ready!(self.mbox.poll_ready(cx)) {
                    Ok(_) => {
                        cx.waker().wake_by_ref();

                        tracing::debug!("fragments messagebox ready, fetching peer address");

                        let fut = self.global_state.peers.get_peer_addr(self.node_id);

                        (
                            FragmentProcessorSendFragmentsState::GetIngressAddress { fut },
                            Poll::Pending,
                        )
                    }
                    Err(e) => {
                        tracing::error!(reason = %e, "error sending fragments for processing");

                        (
                            FragmentProcessorSendFragmentsState::WaitingMessageBox,
                            Poll::Ready(Err(Error::new(Code::Internal, e))),
                        )
                    }
                }
            }
            FragmentProcessorSendFragmentsState::GetIngressAddress { mut fut } => {
                match fut.poll_unpin(cx) {
                    Poll::Ready(addr) => match addr {
                        Some(ingress_addr) => {
                            cx.waker().wake_by_ref();

                            tracing::debug!(
                                ingress_address = %ingress_addr,
                                node_id = %self.node_id,
                                "got ingress address for peer"
                            );

                            // Clone the reference counted global state and move it into a async block
                            // so we can satisfy the 'static lifetime requirement for the reverse lookup future.
                            let global_state = self.global_state.clone();
                            let fut = async move {
                                global_state
                                    .dns_resolver
                                    .reverse_lookup(ingress_addr.ip())
                                    .await
                            }
                            .boxed();

                            (
                                FragmentProcessorSendFragmentsState::ReverseLookup {
                                    ingress_addr,
                                    fut,
                                },
                                Poll::Pending,
                            )
                        }
                        None => {
                            self.clear_buffered_fragments();

                            tracing::warn!(
                                node_id = %self.node_id,
                                "dropping fragments from peer: unable to get address of ingress client"
                            );

                            (
                                FragmentProcessorSendFragmentsState::WaitingMessageBox,
                                Poll::Ready(Ok(())),
                            )
                        }
                    },
                    Poll::Pending => (
                        FragmentProcessorSendFragmentsState::GetIngressAddress { fut },
                        Poll::Pending,
                    ),
                }
            }
            FragmentProcessorSendFragmentsState::ReverseLookup {
                ingress_addr,
                mut fut,
            } => match fut.poll_unpin(cx) {
                Poll::Ready(res) => {
                    cx.waker().wake_by_ref();
                    match res {
                        Ok(lookup) => {
                            // Resolved names come in as absolute FQDN.
                            // We strip that because hostnames are not specified that way.
                            let resolved_hostnames = lookup
                                .iter()
                                .map(|name| name.to_string().trim_end_matches('.').to_string())
                                .collect::<Vec<_>>();

                            tracing::debug!(names = ?resolved_hostnames, "resolved DNS names for peer");

                            (
                                FragmentProcessorSendFragmentsState::SendFragments {
                                    ingress_addr,
                                    resolved_hostnames,
                                },
                                Poll::Pending,
                            )
                        }
                        Err(e) => {
                            tracing::error!(
                                node_id = %self.node_id,
                                address = %ingress_addr,
                                error = ?e,
                                "failed to execute reverse DNS lookup for address"
                            );

                            (
                                FragmentProcessorSendFragmentsState::SendFragments {
                                    ingress_addr,
                                    resolved_hostnames: Vec::new(),
                                },
                                Poll::Pending,
                            )
                        }
                    }
                }
                Poll::Pending => (
                    FragmentProcessorSendFragmentsState::ReverseLookup { ingress_addr, fut },
                    Poll::Pending,
                ),
            },
            FragmentProcessorSendFragmentsState::SendFragments {
                ingress_addr,
                resolved_hostnames,
            } => {
                let should_send = match self.global_state.config.whitelist.as_ref() {
                    Some(whitelist) => {
                        tracing::debug!(
                            "whitelist configured, checking whether fragments should be sent"
                        );

                        whitelist.iter().any(|ma| {
                            match ma.iter().next() {
                                Some(multiaddr::Protocol::Ip4(addr)) => addr == ingress_addr.ip(),
                                Some(multiaddr::Protocol::Ip6(addr)) => addr == ingress_addr.ip(),
                                Some(multiaddr::Protocol::Dns4(fqdn))
                                | Some(multiaddr::Protocol::Dns6(fqdn)) => {
                                    tracing::debug!(resolved_hostnames = ?resolved_hostnames, fqdn = %fqdn, "checking whitelist for domain");

                                    resolved_hostnames.iter().any(|x| x == &fqdn)
                                }
                                _ => {
                                    // In this case the whitelisted multiaddr is invalid
                                    // so consider it as not matching anything
                                    tracing::error!(multiaddr = %ma, "Invalid entry in whitelist");
                                    false
                                }
                            }
                        })
                    }
                    None => {
                        tracing::debug!("no whitelist configured, bypassing check");

                        true
                    }
                };

                if should_send {
                    let fragments = self.clear_buffered_fragments();

                    let addr = match self.global_state.config.address() {
                        Some(addr) => FragmentOrigin::Network { addr: addr.ip() },
                        None => {
                            tracing::info!(
                                "node addr not present in config, reverting to local lookup"
                            );
                            FragmentOrigin::Network {
                                addr: retrieve_local_ip(),
                            }
                        }
                    };

                    let (reply_handle, _reply_future) = intercom::unary_reply();
                    self.mbox
                        .start_send(TransactionMsg::SendTransactions {
                            origin: addr,
                            fragments,
                            fail_fast: false,
                            reply_handle,
                        })
                        .map_err(|e| {
                            tracing::error!(
                                reason = %e,
                                "failed to send fragments to the fragment task"
                            );
                            Error::new(Code::Internal, e)
                        })?;
                    self.refresh_stat();

                    tracing::debug!("processed fragments from peer");

                    (
                        FragmentProcessorSendFragmentsState::WaitingMessageBox,
                        Poll::Ready(Ok(())),
                    )
                } else {
                    self.clear_buffered_fragments();

                    tracing::info!(address = %ingress_addr, "dropping fragments from peer");

                    (
                        FragmentProcessorSendFragmentsState::WaitingMessageBox,
                        Poll::Ready(Ok(())),
                    )
                }
            }
        };

        self.send_fragments_state = next_st;
        ret
    }

    fn poll_flush_mbox(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.mbox).poll_flush(cx).map_err(|e| {
            tracing::error!(
                reason = %e,
                "communication channel to the fragment task failed"
            );
            Error::new(Code::Internal, e)
        })
    }

    fn poll_complete_refresh_stat(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.pending_processing.poll_complete(cx)
    }

    fn clear_buffered_fragments(&mut self) -> Vec<Fragment> {
        mem::replace(
            &mut self.buffered_fragments,
            Vec::with_capacity(buffer_sizes::inbound::FRAGMENTS),
        )
    }
}

impl Sink<net_data::Fragment> for FragmentProcessor {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        if self.buffered_fragments.len() >= buffer_sizes::inbound::FRAGMENTS {
            ready!(self.poll_send_fragments(cx))?;
            debug_assert!(self.buffered_fragments.is_empty());
        }
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, raw_fragment: net_data::Fragment) -> Result<(), Error> {
        assert!(
            self.buffered_fragments.len() < buffer_sizes::inbound::FRAGMENTS,
            "should call `poll_ready` which returns `Poll::Ready(Ok(()))` before `start_send`",
        );
        let fragment = raw_fragment.decode().inspect_err(|e| {
            tracing::info!(
                reason = %e.source().unwrap(),
                "failed to decode incoming fragment"
            );
        })?;
        tracing::debug!(hash = %fragment.hash(), "received fragment");

        self.buffered_fragments.push(fragment);

        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        if self.buffered_fragments.is_empty() {
            if self.pending_processing.complete() {
                tracing::debug!("flushing fragments messagebox");
                self.poll_flush_mbox(cx)
            } else {
                tracing::debug!("waiting pending processing before flushing fragments messagebox");
                let _ = self.poll_complete_refresh_stat(cx);

                Poll::Pending
            }
        } else {
            tracing::debug!("flushing buffered fragments to messagebox");
            ready!(self.poll_send_fragments(cx))?;

            Poll::Pending
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        loop {
            if self.buffered_fragments.is_empty() {
                ready!(self.poll_complete_refresh_stat(cx));
                return Pin::new(&mut self.mbox).poll_close(cx).map_err(|e| {
                    tracing::warn!(
                        reason = %e,
                        "failed to close communication channel to the fragment task"
                    );
                    Error::new(Code::Internal, e)
                });
            } else {
                ready!(self.poll_send_fragments(cx))?;
            }
        }
    }
}

impl Sink<net_data::Gossip> for GossipProcessor {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        ready!(self.pending_processing.poll_complete(cx));
        Ok(()).into()
    }

    fn start_send(mut self: Pin<&mut Self>, gossip: net_data::Gossip) -> Result<(), Error> {
        let nodes = gossip.nodes.decode().inspect_err(|e| {
            tracing::info!(
                reason = %e.source().unwrap(),
                "failed to decode incoming gossip"
            );
        })?;
        tracing::debug!("received gossip on {} nodes", nodes.len());
        let (nodes, filtered_out): (Vec<_>, Vec<_>) = nodes
            .into_iter()
            .partition(|node| filter_gossip_node(node, &self.global_state.config));
        if !filtered_out.is_empty() {
            tracing::debug!("nodes dropped from gossip: {:?}", filtered_out);
        }
        let peer_promoted = std::mem::replace(&mut self.peer_promoted, true);
        let state1 = self.global_state.clone();
        let mut mbox = self.mbox.clone();
        let node_id = self.node_id;

        let fut = future::join(
            async move {
                let refreshed = state1.peers.refresh_peer_on_gossip(&node_id).await;
                if !refreshed {
                    tracing::debug!("received gossip from node that is not in the peer map",);
                }
            },
            async move {
                mbox.send(TopologyMsg::AcceptGossip(nodes.into()))
                    .await
                    .unwrap_or_else(|err| {
                        tracing::error!("cannot send gossips to topology: {}", err)
                    });
                if !peer_promoted {
                    tracing::info!(%node_id, "promoting peer");
                    mbox.send(TopologyMsg::PromotePeer(node_id))
                        .await
                        .unwrap_or_else(|e| {
                            tracing::error!("Error sending message to topology task: {}", e)
                        });
                }
            },
        )
        .in_current_span()
        .map(|_| ());
        self.pending_processing.start(fut);
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        ready!(self.pending_processing.poll_complete(cx));
        Ok(()).into()
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        ready!(self.pending_processing.poll_complete(cx));
        Ok(()).into()
    }
}

#[derive(Default)]
struct PendingProcessing(Option<BoxFuture<'static, ()>>);

impl PendingProcessing {
    fn start(&mut self, future: impl Future<Output = ()> + Send + 'static) {
        self.0 = Some(future.boxed());
    }

    fn poll_complete(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Some(fut) = &mut self.0 {
            ready!(Pin::new(fut).poll(cx));
            self.0 = None;
        }
        Poll::Ready(())
    }

    fn complete(&self) -> bool {
        self.0.is_none()
    }
}
