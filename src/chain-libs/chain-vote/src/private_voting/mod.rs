//! Implementation of the Unit Vector ZK argument presented by
//! Zhang, Oliynykov and Balogum in
//! ["A Treasury System for Cryptocurrencies: Enabling Better Collaborative Intelligence"](https://www.ndss-symposium.org/wp-content/uploads/2019/02/ndss2019_02A-2_Zhang_paper.pdf).
//! We use the notation presented in the technical
//! [spec](https://github.com/input-output-hk/treasury-crypto/blob/master/docs/voting_protocol_spec/Treasury_voting_protocol_spec.pdf),
//! written by Dmytro Kaidalov.

pub(crate) mod challenge_context;
pub(crate) mod messages;
pub mod unit_vector_zkp;

pub(crate) use self::{
    challenge_context::ChallengeContext, messages::Announcement, unit_vector_zkp::Proof,
};
