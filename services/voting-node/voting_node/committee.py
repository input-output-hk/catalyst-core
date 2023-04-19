"""Committee of members that will tally the votes."""
from pydantic import BaseModel


class WalletKeys(BaseModel):
    """The keys to an external wallet.

    `hex_encoded` is used to generate the genesis block.
    """

    seckey: str | None
    pubkey: str
    hex_encoded: str


class Keypair(BaseModel):
    """A pair of key."""

    seckey: str
    """ Secret key."""

    pubkey: str
    """ Public key."""


class CommunicationKeys(Keypair):
    """Committee member communication keys."""


class MemberKeys(Keypair):
    """Committee member keys."""


class CommitteeMember(BaseModel):
    """Committee member."""

    index: int
    """Zero-based index for this committee member."""
    communication_keys: CommunicationKeys
    """Member communication keypair."""
    member_keys: MemberKeys
    """Committee member keypair."""


class ElectionKey(BaseModel):
    """The election key is used to sign every vote.

    This key can be rebuilt with the committee member keys.
    """

    pubkey: str
