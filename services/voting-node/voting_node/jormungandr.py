class Jormungandr(object):
    """Wrapper type for the jormungandr command-line."""

    def __init__(self, jorm_exec: str):
        self.jorm_exec = jorm_exec
