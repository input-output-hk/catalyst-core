"""Committee of members that will tally the votes."""
import yaml
from aiofile import async_open
from loguru import logger
from pathlib import Path
from pydantic import BaseModel
from typing import Self


class WalletKeys(BaseModel):
    """The keys to an external wallet."""

    seckey: str
    """`seckey` is the secret key for the wallet."""
    pubkey: str
    """`pubkey` is the public key for the wallet."""
    hex_encoded: str
    """`hex_encoded` is used to generate the genesis block."""


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
    """ Public key."""


class Committee(BaseModel):
    """The tallying committee."""

    row_id: int | None = None
    """`row_id` the unique key for this committee in the DB."""
    event_id: int
    """`event_id` the number of committee members."""
    size: int
    """`size` the number of committee members."""
    threshold: int
    """`threshold` the minimum number of members needed to tally."""
    crs: str
    """`crs` the encrypted Common Reference String shared in the creation of every set of committee member keys."""
    committee_pk: str
    """`committee_pk` the encrypted private key of the Committee address."""
    committee_id: str
    """`committee_id` the hex-encoded public key of the Committee address."""
    members: list[CommitteeMember] | None = None
    """`members` list of containing the communication and member secrets of each member of the commitee."""
    election_key: ElectionKey
    """`election_key` public key used to sign every vote in the event. This key is created from the committee member public keys."""

    def as_yaml(self) -> str:
        """Return the content as YAML."""
        return yaml.safe_dump(self.dict())

    @classmethod
    async def read_file(cls, file: Path) -> Self:
        """Read and return the yaml_type from the file path."""
        afp = await async_open(file, "r")
        yaml_str = await afp.read()
        await afp.close()
        yaml_dict = yaml.safe_load(yaml_str)
        try:
            members_list = yaml_dict["members"]

            def committee_member(member: dict) -> CommitteeMember:
                comm_keys = [print(keys) for keys in member["communication_keys"]]
                comm_keys = [CommunicationKeys(**keys) for keys in member["communication_keys"]]
                logger.debug(f"comm_keys: {comm_keys}")
                member["communication_keys"] = comm_keys
                member_keys = [MemberKeys(**keys) for keys in member["member_keys"]]
                member["member_keys"] = member_keys
                return CommitteeMember(**member)

            yaml_dict["members"] = [committee_member(member) for member in members_list]
            committee = cls(**yaml_dict)
            return committee
        except Exception as e:
            raise Exception(f"invalid committee in {file}: {e}")
