"""Wrapper for the jcli command-line executable."""
import asyncio
from pathlib import Path

from .models.committee import ElectionKey


class JCli:
    """Wrapper type for the jcli command-line."""

    def __init__(self, jcli_exec: str) -> None:
        """Initialize by setting the path string to the jcli executable."""
        self.jcli_exec = jcli_exec

    async def key_generate(self, secret_type: str = "ed25519") -> str:
        """Return a private (secret) key from the given secret_type.

        Possible values: "ed25519", "ed25519-bip32", "ed25519-extended",
        "sum-ed25519-12", "ristretto-group2-hash-dh".

        Defaults to "ed25519".
        """
        # run jcli to generate the secret key
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            "key",
            "generate",
            "--type",
            secret_type,
            stdout=asyncio.subprocess.PIPE,
        )
        # checks that there is stdout
        if proc.stdout is None:
            raise Exception("failed to generate secret")
        # read the output
        data = await proc.stdout.readline()
        if data is None:
            raise Exception("failed to generate secret")
        # get the key and store it in the file
        key = data.decode().rstrip()
        return key

    async def key_to_public(self, seckey: str) -> str:
        """Return a public key the given secret key string."""
        # run jcli to generate the secret key
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            "key",
            "to-public",
            stdout=asyncio.subprocess.PIPE,
            stdin=asyncio.subprocess.PIPE,
        )

        stdout, _ = await proc.communicate(input=seckey.encode())
        # checks that there is stdout
        if stdout is None:
            raise Exception("failed to generate secret")
        # read the output
        key = stdout.decode().rstrip()
        return key

    async def key_to_bytes(self, key: str) -> str:
        """Return the hex-encoded bytes of a given key string."""
        # run jcli to generate the secret key
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            "key",
            "to-bytes",
            stdout=asyncio.subprocess.PIPE,
            stdin=asyncio.subprocess.PIPE,
        )

        stdout, _ = await proc.communicate(input=key.encode())
        # checks that there is stdout
        if stdout is None:
            raise Exception("failed to generate secret")
        # read the output
        key = stdout.decode().rstrip()
        return key

    async def votes_committee_communication_key_generate(self) -> str:
        """Run 'jcli genesis encode' to make block0 from genesis.yaml."""
        proc_args = (
            "votes",
            "committee",
            "communication-key",
            "generate",
        )
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            *proc_args,
            stdout=asyncio.subprocess.PIPE,
        )
        # checks that there is stdout
        stdout, _ = await proc.communicate()
        if stdout is None:
            raise Exception("failed to generate committee communication key")
        # read the output
        commkey = stdout.decode().rstrip()
        return commkey

    async def votes_committee_communication_key_to_public(self, input_key: str) -> str:
        """Run 'jcli vote committee communication-key to-public [INPUT]' to return the public communication key."""
        proc_args = (
            "votes",
            "committee",
            "communication-key",
            "to-public",
        )
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            *proc_args,
            stdout=asyncio.subprocess.PIPE,
            stdin=asyncio.subprocess.PIPE,
        )
        # checks that there is stdout
        stdout, _ = await proc.communicate(input=input_key.encode())
        if stdout is None:
            raise Exception("failed to generate committee public communication key")
        # read the output
        commid = stdout.decode().rstrip()
        return commid

    async def votes_committee_member_key_generate(self, comm_pks: list[str], crs: str, index: int, threshold: int) -> str:
        """Run 'jcli vote committee member-key to-public [INPUT]' to return the public communication key."""
        keys_args = ()
        for comm_pk in comm_pks:
            keys_args = (*keys_args, "--keys", comm_pk)

        proc_args = (
            "votes",
            "committee",
            "member-key",
            "generate",
            "--threshold",
            f"{threshold}",
            "--crs",
            crs,
            "--index",
            f"{index}",
            *keys_args,
        )
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            *proc_args,
            stdout=asyncio.subprocess.PIPE,
        )
        # checks that there is stdout
        stdout, _ = await proc.communicate()
        if stdout is None:
            raise Exception("failed to generate committee member key")
        # read the output
        membersk = stdout.decode().rstrip()
        return membersk

    async def votes_committee_member_key_to_public(self, member_sk: str) -> str:
        """Run 'jcli vote committee member-key to-public [INPUT]' to return the public communication key."""
        proc_args = (
            "votes",
            "committee",
            "member-key",
            "to-public",
        )
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            *proc_args,
            stdout=asyncio.subprocess.PIPE,
            stdin=asyncio.subprocess.PIPE,
        )
        # checks that there is stdout
        stdout, _ = await proc.communicate(input=member_sk.encode())
        if stdout is None:
            raise Exception("failed to generate committee member public key")
        # read the output
        memberpk = stdout.decode().rstrip()
        return memberpk

    async def votes_election_key(self, member_pks: list[str]) -> ElectionKey:
        """Run 'jcli vote election-key --keys [PUBLIC_MEMBER_KEYS]' to return the election key."""
        keys_args = ()
        for member_pk in member_pks:
            keys_args = (*keys_args, "--keys", member_pk)
        proc_args = ("votes", "election-key", *keys_args)
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            *proc_args,
            stdout=asyncio.subprocess.PIPE,
        )
        # checks that there is stdout
        stdout, _ = await proc.communicate()
        if stdout is None:
            raise Exception("failed to generate committee member public key")
        # read the output
        memberpk = stdout.decode().rstrip()
        return ElectionKey(pubkey=memberpk)

    async def genesis_encode(self, block0_bin: Path, genesis_yaml: Path):
        """Run 'jcli genesis encode' to make block0 from genesis.yaml."""
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            "genesis",
            "encode",
            "--input",
            f"{genesis_yaml}",
            "--output",
            f"{block0_bin}",
            stdout=asyncio.subprocess.PIPE,
        )

        returncode = await proc.wait()
        # checks that the subprocess did not fail
        if returncode > 0:
            raise Exception("failed to generate block0")

    async def genesis_hash(self, block0_bin: Path) -> str:
        """Run 'jcli genesis hash' to get the hash from block0.bin."""
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            "genesis",
            "hash",
            "--input",
            f"{block0_bin}",
            stdout=asyncio.subprocess.PIPE,
        )

        # checks that there is stdout
        stdout, _ = await proc.communicate()
        if stdout is None:
            raise Exception("failed to generate block0 hash")
        # read the output
        hash = stdout.decode().rstrip()
        return hash

    async def genesis_decode(self, block0_bin: Path, genesis_yaml: Path) -> None:
        """Run 'jcli genesis decode' on block0.bin and saves to genesis.yaml."""
        proc = await asyncio.create_subprocess_exec(
            self.jcli_exec,
            "genesis",
            "decode",
            "--input",
            f"{block0_bin}",
            "--output",
            f"{genesis_yaml}",
            stdout=asyncio.subprocess.PIPE,
        )

        returncode = await proc.wait()
        # checks that the subprocess did not fail
        if returncode > 0:
            raise Exception("failed to decode block0")
