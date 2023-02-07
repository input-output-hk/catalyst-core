import logging

def getLogger(log_level):
    logger = logging.getLogger('voting-node')
    logger.setLevel(getattr(logging, log_level.upper()))
    ch = logging.StreamHandler()
    ch.setLevel(getattr(logging, log_level.upper()))
    formatter = logging.Formatter('%(asctime)s - %(levelname)s - %(name)s - %(message)s')
    ch.setFormatter(formatter)
    logger.addHandler(ch)
    return logger
