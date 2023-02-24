# Stage 1
FROM jormungandr:latest


# Stage 2
FROM python:3.10-slim-bullseye
WORKDIR /voting

# Add python codebase
COPY . /voting

RUN pip3 install .

# Stage 2: start the service
CMD ["voting-node", "start", "--host", "0.0.0.0", "--port", "5057"]
COPY --from=0 /usr/local/bin/jormungandr /usr/local/bin/jormungandr
COPY --from=0 /usr/local/bin/jcli /usr/local/bin/jcli
