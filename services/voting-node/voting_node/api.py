import prometheus_fastapi_instrumentator
from fastapi import FastAPI
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor

app = FastAPI()


@app.get("/")
def heartbeat():
    """Returns 200 if the service is running."""
    return None


FastAPIInstrumentor.instrument_app(app)
prometheus_fastapi_instrumentator.Instrumentator().instrument(app).expose(app)
