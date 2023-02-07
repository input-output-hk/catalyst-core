# Stage 1
FROM jormungandr:latest

WORKDIR /voting

## Update container and copy executables
RUN apt-get update && \
    apt-get install -y python3-pip

## apt cleanup
RUN apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Add python codebase
COPY . /voting

RUN pip3 install .

# Stage 2: start the service
CMD ["voting-node", "start", "--host", "0.0.0.0", "--port", "5057"]
