"""Module to compute similarity between texts."""
from typing import List, Iterable
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity

from review_stage.db import models
from review_stage import utils
from review_stage.tools.importer import Importer

from rich import print

import numpy as np


class Similarity:
    """Performs the similarity analysis."""

    def __init__(self, importer: Importer, similarity_threshold: float = 0.99):
        """Initialize entities."""
        self.data = importer

        self.vectorize = lambda Text: TfidfVectorizer().fit_transform(Text).toarray()
        self.batch_size = 1500
        self.complete_batch_size = 15000
        self.similarity_threshold = similarity_threshold
        self.criteria = ["impact_note", "auditability_note", "feasibility_note"]

    def __get_fund_reviews(self, fund: int) -> Iterable[models.Review]:
        return filter(lambda a: a.fund == fund, self.data.reviews)

    def __associate_mappings(
        self, mapping: List[dict], matrix: List[float], indexes: np.ndarray, x: int, y: int
    ) -> List[models.SimilarPair]:
        results = []
        for coords in indexes:
            left = mapping[coords[0] + (x * self.batch_size)]
            right = mapping[coords[1] + (y * self.complete_batch_size)]
            if not (
                (left["review"].id == right["review"].id)
                and (left["review"].fund == right["review"].fund)
                and (left["criterium"] == right["criterium"])
            ):
                results.append(
                    models.SimilarPair(
                        left=left["review"],
                        right=right["review"],
                        left_criterium=left["criterium"],
                        right_criterium=right["criterium"],
                        score=matrix[coords[0]][coords[1]],
                    )
                )
        return results

    def generate_similarity(self, current_fund: int) -> List[models.SimilarPair]:
        """Generate the similarity comparing vectors with cosine similarity."""
        results: List[models.SimilarPair] = []
        print("[bold yellow]Prepare vectors...[/bold yellow]")
        notes, mapping = self.data.prepare_reviews(self.criteria)
        vectors = self.vectorize(notes)

        print("[bold yellow]Batch vectors...[/bold yellow]")
        current_fund_reviews = self.__get_fund_reviews(current_fund)
        current_fund_tot_reviews = len(list(current_fund_reviews))
        current_fund_vector_max = current_fund_tot_reviews * len(self.criteria)
        print(f"Total reviews: {len(self.data.reviews)}")
        print(f"Total notes: {len(notes)}")
        print(f"Total current fund reviews: {current_fund_tot_reviews}")
        print(f"Total current fund notes: {current_fund_vector_max}")
        chunks = list(utils.batch(vectors[0:current_fund_vector_max], self.batch_size))
        complete_chunks = list(utils.batch(vectors, self.complete_batch_size))
        for x, chunk in enumerate(chunks):
            for y, complete_chunk in enumerate(complete_chunks):
                print(f"[bold yellow]Compute similarity for batch {x},{y}...[/bold yellow]")
                similarity_matrix = cosine_similarity(chunk, complete_chunk)
                indexes = np.argwhere(similarity_matrix > self.similarity_threshold)
                results = results + self.__associate_mappings(mapping, similarity_matrix, indexes, x, y)
        return results

    def export_results(self, path, results: List[models.SimilarPair], current_fund: int):
        """Display the results of similarity analysis."""
        print(f"Total similarities: {len(results)}")
        if len(results) > 0:
            utils.deserialize_and_save_csv(
                f"{path}/similarity.csv",
                results,
                {"left": {"id": True}, "right": {"id": True}, "score": True, "left_criterium": True, "right_criterium": True},
                None,
            )
            involved_assessments = []
            ids = []
            for result in results:
                if result.left.fund != current_fund:
                    if result.left.id not in ids:
                        involved_assessments.append(result.left)
                        ids.append(result.left.id)
                if result.right.fund != current_fund:
                    if result.right.id not in ids:
                        involved_assessments.append(result.right)
                        ids.append(result.right.id)
            
            if len(involved_assessments) > 0:
                utils.deserialize_and_save_csv(
                    f"{path}/similarity_involved_assessments.csv",
                    involved_assessments,
                    {
                        "id": True,
                        "assessor": True,
                        "impact_note": True,
                        "impact_rating": True,
                        "feasibility_note": True,
                        "feasibility_rating": True,
                        "auditability_note": True,
                        "auditability_rating": True,
                        "proposal_id": True,
                        "fund": True
                    },
                    None
                ),
                
        """
        for result in results:
            print(result.score)
            print(result.left)
            print(result.right)
            print(result.left.id, result.right.id)
            print(result.left_criterium, result.right_criterium)
            print(getattr(result.left, result.left_criterium))
            print("====")
            print(getattr(result.right, result.right_criterium))
            print("####")
        """
