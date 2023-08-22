"""Module to postprocess reviews."""
from typing import List

from ..types import models
from .. import utils
from .importer import Importer


class Postprocessor:
    """Module to manipulate and postprocess reviews."""

    def __init__(self, importer: Importer):
        """Initialize entities."""
        self.data = importer

    def postprocess_reviews(self) -> List[models.Review]:
        """Anonymize, assign level and allocation to reviews."""
        anonymized_map = {}
        for _a in self.data.allocations:
            if _a.pa_email not in anonymized_map:
                anonymized_map[_a.pa_email] = _a
                self.data.pas.append(_a)
        for review in self.data.reviews:
            _allocation = next(
                (_a for _a in self.data.allocations if (_a.pa_email == review.assessor and _a.proposal_id == review.proposal.id)),
                None,
            )
            if _allocation:
                review.allocated = True
            else:
                review.allocated = False
            if review.assessor in anonymized_map:
                review.level = anonymized_map[review.assessor].pa_level
                review.assessor = anonymized_map[review.assessor].pa_anon_id
            else:
                review.level = 0
        # return [review.anonymize() for review in self.reviews]

    def export_pas(self, path: str):
        """Export the PAs active."""
        utils.deserialize_and_save_csv(
            path,
            self.data.pas,
            {
                "pa_anon_id": True,
                "pa_email": True,
                "pa_level": True,
            },
            "",
        )

    def export_reviews(self, source, path: str):
        """Export the postprocessed reviews to CSV."""
        utils.deserialize_and_save_csv(
            path,
            source,
            {
                "id": True,
                "assessor": True,
                "impact_note": True,
                "impact_rating": True,
                "feasibility_note": True,
                "feasibility_rating": True,
                "auditability_note": True,
                "auditability_rating": True,
                "level": True,
                "allocated": True,
                "proposal": {"id", "url", "title", "challenge_id"},
            },
            "postprocessed_reviews",
        )
