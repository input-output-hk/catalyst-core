"""Module to detect ai generated content."""
import json
from itertools import groupby
from typing import List, Iterable, Mapping, Any, cast

from review_stage.db import models
from review_stage.tools.importer import Importer
from review_stage import utils

import asyncio


class GptZero:
    """Interface with GptZero services."""

    def __init__(self, importer: Importer, api_key: str, ai_threshold: float = 0.65):
        """Initialize entities."""
        self.data = importer

        self.api_key = api_key
        self.api_url = "https://api.gptzero.me"
        self.predict_endpoint = "/v2/predict/text"
        self.inner = utils.JsonHttpClient(self.api_url)
        self.criteria = ["impact_note", "auditability_note", "feasibility_note"]
        self.ai_threshold = ai_threshold
        self.N_WORKERS = 10

        self.ai_cache_path = "data/ai_cache.json"
        self.load_cache()

    def load_cache(self):
        """Load a local cache of requests."""
        self.cache = []
        print("Loading translations cache")
        with open(self.ai_cache_path, "r") as file:
            self.cache = json.load(file)

    def save_cache(self):
        """Save a local cache of requests."""
        print("Saving translations cache")
        with open(self.ai_cache_path, "w") as file:
            file.write(json.dumps(self.cache))

    def search_cache(self, text):
        """Search in cache for a request."""
        cached = [c for c in self.cache if c["text"] == text]
        if len(cached) > 0:
            print("hit cache")
            return cached[0]["response"]
        else:
            return False

    def store_cache(self, detection: models.AiDetection, raw_response):
        """Store a request in cache."""
        if raw_response:
            self.cache.append({"text": detection.review.full_note, "response": raw_response})
            if (len(self.cache) % 20) == 0:
                self.save_cache()

    async def _post(self, review: models.Review) -> models.AiDetection:
        cached = self.search_cache(review.full_note)
        if cached:
            documents = cast(Mapping[str, Any], cached)["documents"]
            return models.AiDetection(review=review, **documents[0]), None
        else:
            headers = {"X-Api-Key": self.api_key}
            body = {"document": review.full_note}
            response = await self.inner.post(self.predict_endpoint, body, headers)
            documents = cast(Mapping[str, Any], response)["documents"]
            return models.AiDetection(review=review, **documents[0]), response

    def detect_ai(self) -> List[models.AiDetection]:
        """Predict reviews that contains AI generated content."""
        predictions = []

        async def inner():
            tasks: asyncio.Queue = asyncio.Queue()
            for review in self.data.reviews:
                tasks.put_nowait(self._post(review))

            async def worker():
                while not tasks.empty():
                    res, raw = await tasks.get_nowait()
                    predictions.append(res)
                    self.store_cache(res, raw)

            await asyncio.gather(*[worker() for _ in range(self.N_WORKERS)])

        asyncio.run(inner())

        # Todo: save cache and filter predictions.
        self.save_cache()
        ai_generated = self._filter_predictions(predictions)
        return ai_generated

    def _filter_predictions(self, predictions: List[models.AiDetection]) -> Iterable[models.AiDetection]:
        return list(filter(lambda el: el.generated_prob >= self.ai_threshold, predictions))

    def export_results(self, path: str, results):
        """Export the results of the analysis."""
        if len(results) > 0:
            utils.deserialize_and_save_csv(
                path, results, {"review": {"id": True}, "generated_prob": True, "avg_generated_prob": True}, None
            )

    def detect_by_reviewer(self) -> List[models.AiDetection]:
        """Predict reviews that contains AI generated content by reviewer."""
        samples = []
        self.data.reviews.sort(key=lambda x: x.assessor)
        for assessor, items in groupby(self.data.reviews, key=lambda x: x.assessor):
            samples.append(list(items)[0])

        self.data.reviews = samples
