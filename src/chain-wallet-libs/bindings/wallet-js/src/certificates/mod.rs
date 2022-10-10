use chain_impl_mockchain::certificate::Certificate as CertificateLib;

pub mod vote_cast;

pub struct Certificate(pub(crate) CertificateLib);

impl From<vote_cast::VoteCast> for Certificate {
    fn from(val: vote_cast::VoteCast) -> Self {
        Self(CertificateLib::VoteCast(val.0))
    }
}
