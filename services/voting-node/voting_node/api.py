from fastapi import FastAPI
from prometheus_fastapi_instrumentator import Instrumentator
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor

app = FastAPI()


@app.get("/")
def heartbeat():
    """Returns 200 if the service is running."""
    return None


Instrumentator().instrument(app).expose(app)
FastAPIInstrumentor.instrument_app(app)
