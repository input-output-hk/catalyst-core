# stage 1
FROM python:3.10-slim-bullseye as python
ENV PYTHONUNBUFFERED=true
WORKDIR /voting

# stage 2
FROM jormungandr:latest as jorm

# stage 3
FROM python as poetry
RUN pip install poetry

# Add python codebase
COPY . /voting
RUN poetry shell; poetry install

# final stage
FROM python
ENV PATH="/voting/.venv/bin:/usr/local/bin:$PATH"
COPY --from=poetry /voting /voting
COPY --from=jorm /usr/local/bin/jormungandr /usr/local/bin/jormungandr
COPY --from=jorm /usr/local/bin/jcli /usr/local/bin/jcli
EXPOSE 5057
CMD voting-node start --host 0.0.0.0 --port 5057
