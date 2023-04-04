"""FastAPI server for the voting node."""
import prometheus_fastapi_instrumentator
from fastapi import FastAPI
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor

app = FastAPI()


@app.get("/")
def heartbeat():
    """Return 200 if the service is running."""
    return


FastAPIInstrumentor.instrument_app(app)
prometheus_fastapi_instrumentator.Instrumentator().instrument(app).expose(app)
