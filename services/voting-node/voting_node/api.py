from prometheus_fastapi_instrumentator import Instrumentator
from fastapi import FastAPI

app = FastAPI()


@app.get("/")
def heartbeat():
    """Returns 200 if the service is running."""
    return None


Instrumentator().instrument(app).expose(app)
