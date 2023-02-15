# Configuration files go there


class JormConfig(object):
    """ Holds parameters used to configure and start jormungandr."""
    def __init__(
        self, jormungandr_path: str, jcli_path: str, storage: str, node_type: str
    ):
        self.jormungandr_path = jormungandr_path
        self.jcli_path = jcli_path
        self.storage = storage
        self.node_type = node_type
