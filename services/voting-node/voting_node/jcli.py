import asyncio


class JCli(object):
    """Wrapper type for the jcli command-line."""

    def __init__(self, jcli_exec: str):
        self.jcli_exec = jcli_exec

    async def seckey(self, secret_type: str) -> str:
        """Returns ed25519 secret key."""
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
        """Returns ed25519 public key."""
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
