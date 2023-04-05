import asyncio
from pathlib import Path


class JCli(object):
    """Wrapper type for the jcli command-line."""

    def __init__(self, jcli_exec: str):
        self.jcli_exec = jcli_exec

    async def privkey(self, secret_type: str = "ed25519") -> str:
        """Returns a private (secret) key from the given secret_type.

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

    async def pubkey(self, seckey: str) -> str:
        """Returns a public key the given secret key string."""
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

    async def key_to_hex(self, key: str) -> str:
        """Returns the hex-encoded bytes of a given key string."""
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

    async def genesis_encode(self, block0_bin: Path, genesis_yaml: Path):
        """Run 'jcli genesis encode' to make block0 from genesis.yaml"""
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
        """Run 'jcli genesis hash' to get the hash from block0.bin"""
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
