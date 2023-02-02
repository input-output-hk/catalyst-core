# Stage 1
FROM jormungandr:latest

WORKDIR /voting


## Update container and copy executables
RUN apt-get update && \
    apt-get install -y python3-venv python3-pip


## apt cleanup
RUN apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Add python codebase
COPY requirements.txt requirements.txt
COPY main.py ./

RUN pip install --upgrade pip && \
    pip install --no-cache-dir -r requirements.txt

# Stage 2: start the service
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "5057"]
