import logging


def configLogger(log_level: str):
    """Configures logging for the voting node."""
    logger = logging.getLogger("voting-node")
    logger.setLevel(getattr(logging, log_level.upper()))
    ch = logging.StreamHandler()
    ch.setLevel(getattr(logging, log_level.upper()))
    formatter = logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
    ch.setFormatter(formatter)
    logger.addHandler(ch)


def getLogger() -> logging.Logger:
    """Returns the voting node logger."""
    return logging.getLogger("voting-node")
