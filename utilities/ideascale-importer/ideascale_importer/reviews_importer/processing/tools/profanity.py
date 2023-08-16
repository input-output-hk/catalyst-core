"""Module to compute similarity between texts."""
from typing import List
from profanity_check import predict_prob

from review_stage.db import models
from review_stage.tools.importer import Importer
from review_stage import utils

import numpy as np


class Profanity:
    """Performs the similarity analysis."""

    def __init__(self, importer: Importer, profanity_threshold: float = 0.75):
        """Initialize entities."""
        self.data = importer

        self.profanity_threshold = profanity_threshold
        self.criteria = ["impact_note", "auditability_note", "feasibility_note"]

    def __associate_mappings(self, mapping: List[dict], predictions: List[float], indexes: np.ndarray) -> List[models.Profanity]:
        results = []
        for coords in indexes:
            review = mapping[coords[0]]
            results.append(
                models.Profanity(
                    review=review["review"],
                    criterium=review["criterium"],
                    score=predictions[coords[0]],
                )
            )
        return results

    def check_profanity(self) -> List[models.Profanity]:
        """Predict reviews that contains profanities."""
        notes, mapping = self.data.prepare_reviews(self.criteria)
        predictions = predict_prob(notes)
        indexes = np.argwhere(predictions > self.profanity_threshold)
        results = self.__associate_mappings(mapping, predictions, indexes)
        return results

    def export_results(self, path: str, results: List[models.Profanity]):
        """Export the results of the analysis."""
        if len(results) > 0:
            utils.deserialize_and_save_csv(
                path,
                results,
                {"review": {"id": True}, "score": True, "criterium": True},
                "profanity",
            )
