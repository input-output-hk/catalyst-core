# Configuration files go there


class JormConfig(object):
    """Holds parameters used to configure and start jormungandr."""

    def __init__(
        self,
        jorm_path: str,
        jcli_path: str,
        storage: str,
        rest_port: int,
        jrpc_port: int,
        p2p_port: int,
    ):
        self.jorm_path = jorm_path
        self.jcli_path = jcli_path
        self.storage = storage
        self.rest_port = rest_port
        self.jrpc_port = jrpc_port
        self.p2p_port = p2p_port
