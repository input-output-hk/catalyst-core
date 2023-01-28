FROM python:3-slim-bullseye

WORKDIR /voting

COPY requirements.txt requirements.txt
RUN pip install -r requirements.txt

COPY main.py ./

CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "5057"]
