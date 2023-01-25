FROM python:3.10-alpine

RUN adduser -D voting

WORKDIR /home/voting

COPY requirements.txt requirements.txt
RUN pip install -r requirements.txt

COPY main.py ./

RUN chown -R voting:voting ./
USER voting
CMD uvicorn main:app --host 0.0.0.0 --port 5057
