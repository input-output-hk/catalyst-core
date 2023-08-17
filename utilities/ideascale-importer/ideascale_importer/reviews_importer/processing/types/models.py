"""Models for reviews."""
from __future__ import annotations
from pydantic import BaseModel, HttpUrl, Field, validator, root_validator
from typing import List, Optional
import re


class Model(BaseModel):
    """Base class for all models."""

class Pa(Model):
    """Represents a PA."""

    anon_id: str = Field(alias="ids")
    email: Optional[str] = Field(default=None)
    rewards_address: Optional[str] = Field(default=None)
    challenge_ids: List[int] = Field(default=[])
    level: int = Field(default=0)
    name: Optional[str] = Field(default=None)
    user_name: Optional[str] = Field(default=None)
    id: Optional[int] = Field(default=None)
    allocations: List[Allocation] = Field(default=[])

    @validator("anon_id", pre=True)
    @classmethod
    def parse_anon_id(cls, value):
        """Get anonymized id from a list. The first one is the main that is used."""
        id = value.split(",")[0]
        return id

    @validator("email", pre=True)
    @classmethod
    def lower_email(cls, value):
        """Use lowercase only for emails."""
        return value.lower()

    @root_validator(pre=True)
    @classmethod
    def assign_all_challenges_if_empty(cls, values):
        """Assign challenges_ids when field is not populated."""
        if "challenge_ids" not in values:
            # random.randint(1, len(values["challenges"]))
            # challenges = random.choices(values['challenges'], k=nr_of_choices)
            # values['challenge_ids'] = [el.id for el in challenges]
            if len(values["challenges"]) > 0:
                values["challenge_ids"] = [el.id for el in values["challenges"]]
            else:
                values["challenge_ids"] = []
        return values


class Author(Model):
    """Represents an author."""

    id: int
    name: str
    email: str
    user_name: str


class Challenge(Model):
    """Represents a challenge."""

    id: int
    title: str
    funds: Optional[int] = Field(default=None)


class Proposal(Model):
    """Represents a proposal."""

    id: int
    url: HttpUrl
    title: str
    challenge_id: int = Field(alias="campaign_id")
    times_picked: int = Field(default=0)
    funds: Optional[int]
    allocations: List[Allocation] = Field(default=[])
    authors: List[Author] = Field(default=[])
    public_email: Optional[str]
    stage_id: Optional[int]
    extra: Optional[dict]

    @root_validator(pre=True)
    @classmethod
    def assign_authors_if_any(cls, values):
        """Assign proposers/co-proposers merging different ideascale fields."""
        authors = []
        if "author_info" in values:
            authors.append(Author(**values["author_info"]))
        if "contributors" in values:
            for contributor in values["contributors"]:
                authors.append(Author(**contributor))
        values["authors"] = authors
        values["extra"] = {}
        if "custom_fields_by_key" in values:
            if "f10_main_contact_email" in values["custom_fields_by_key"]:
                values["public_email"] = values["custom_fields_by_key"]["f10_main_contact_email"]
            if "f10_requested_funds" in values["custom_fields_by_key"]:
                amount = values["custom_fields_by_key"]["f10_requested_funds"]
                amount = int(re.sub(r"[^0-9]", "", amount))
                values["funds"] = amount
            else:
                values["funds"] = 0
            values["extra"] = values["custom_fields_by_key"]
        return values

    @property
    def full_text(self):
        """Return the full text combining all the fields of the proposal."""
        return " ".join(self.extra.values())


class Allocation(Model):
    """Represent a proposal allocated to a PA."""

    pa: Pa
    proposal: Proposal


class AllocationLight(Model):
    """Represent a proposal allocated to a PA with light information."""

    pa_anon_id: int
    pa_email: str
    pa_level: int
    proposal_id: int


class Review(Model):
    """Represent an imported Assessment."""

    id: int
    assessor: str = Field(alias="Assessor")
    impact_note: str = Field(alias="Impact / Alignment Note")
    impact_rating: int = Field(alias="Impact / Alignment Rating")
    feasibility_note: str = Field(alias="Feasibility Note")
    feasibility_rating: int = Field(alias="Feasibility Rating")
    auditability_note: str = Field(alias="Auditability Note")
    auditability_rating: int = Field(alias="Auditability Rating")
    classification: Optional[str] = Field(alias="Result")
    level: Optional[int]
    allocated: Optional[bool]
    pa: Optional[Pa]
    proposal: Optional[Proposal]
    fund: Optional[int]
    proposal_id: Optional[int]

    @property
    def full_note(self):
        """Return the full not combining the single criteria notes."""
        return " ".join([self.impact_note, self.feasibility_note, self.auditability_note])


class SimilarPair(Model):
    """Represent a pair of assessments similar."""

    left: Review
    right: Review
    score: float
    left_criterium: str
    right_criterium: str

    @root_validator(pre=True)
    @classmethod
    def lower_always_left(cls, values):
        """Force the lower score to be on `left` field."""
        if values["left"].id > values["right"].id:
            left = values["left"]
            left_criterium = values["left_criterium"]
            values["left"] = values["right"]
            values["left_criterium"] = values["right_criterium"]
            values["right"] = left
            values["right_criterium"] = left_criterium
        return values


class ReviewEmbedding(Model):
    """Represents a vector embedding for a review."""

    review: Review
    criteria: str
    embedding_type: str
    embeddings: List[float]


class Profanity(Model):
    """Represent a review criteria profanity."""

    review: Review
    score: float
    criterium: str


class AiDetectionParagraph(Model):
    """Represent a paragraph analysis in GptZero response."""

    generated_prob: float = Field(alias="completely_generated_prob")
    num_sentences: int
    start_sentence_index: int


class AiDetectionSentence(Model):
    """Represent a sentence analysis in GptZero response."""

    generated_prob: float
    perplexity: int
    sentence: str


class AiDetection(Model):
    """Represent a review criteria ai detection."""

    review: Review
    avg_generated_prob: float = Field(alias="average_generated_prob")
    generated_prob: float = Field(alias="completely_generated_prob")
    burstiness: float = Field(alias="overall_burstiness")
    paragraphs: List[AiDetectionParagraph]
    sentences: List[AiDetectionSentence]


class IdeascaleComRev(Model):
    """Represent a Community Reviewer in Ideascale."""

    id: int = Field(default=0)
    email: str = Field(default="")
    rewards_address: Optional[str] = Field(default=None)
    preferred_challenges: List[str] = Field(default=[])
    subscribed: bool = Field(default=False)
    name: Optional[str] = Field(default=None)
    user_name: Optional[str] = Field(default=None)
    id: Optional[int] = Field(default=None)

    @root_validator(pre=True)
    @classmethod
    def parse_custom_fields(cls, values):
        """Parse custom fields into fields."""
        if "would you like to participate as a reviewer in the community review stage?" in values["profile_questions"]:
            values["subscribed"] = (
                values["profile_questions"]["would you like to participate as a reviewer in the community review stage?"]
                == "Yes, I want to be a Community Reviewer and I also understand the role."
            )
        else:
            values["subscribed"] = False
        if "rewards address" in values["profile_questions"]:
            values["rewards_address"] = values["profile_questions"]["rewards address"]
        if "preferred challenges" in values["profile_questions"]:
            pf = values["profile_questions"]["preferred challenges"].strip()
            if pf == "":
                values["preferred_challenges"] = []
            else:
                values["preferred_challenges"] = [c.strip() for c in pf.split(",")]
        else:
            values["preferred_challenges"] = []
        return values
    
    @validator("email", pre=True)
    @classmethod
    def lower_email(cls, value):
        """Use lowercase only for emails."""
        return value.lower()


class IdeascaleChallenge(Model):
    """Represents a challenge in Ideascale."""

    id: int
    title: str = Field(alias="name")


class IdeascaleExportedReview(Model):
    """Represents a review exported from Ideascale Excel file."""

    idea_id: Optional[int] = Field(default=None)
    idea_title: str = Field(alias="Idea Title", default="")
    idea_url: str = Field(alias="Idea URL", default="")
    idea_challenge: Optional[str] = Field(default=None)
    question: str = Field(alias="Question", default="")
    email: str = Field(alias="Assessor", default="")
    note: str = Field(alias="Assessment Note", default="")
    score: int = Field(1, alias="Rating Given")
    date: str = Field(alias="Date", default="")

    @validator("score", pre=True)
    @classmethod
    def catch_na(cls, value):
        """Catch NA."""
        if value == "NA":
            return 1
        return value

    @validator("email", pre=True)
    @classmethod
    def lower_email(cls, value):
        """Use lowercase only for emails."""
        return value.lower()

    class Config:
        """Extra configuration options."""

        anystr_strip_whitespace = True


class IdeascaleExportedReviewResult(Model):
    """Represents a review exported from Ideascale Excel file."""

    idea_id: int = Field(alias="Idea ID", default=0)
    idea_title: str = Field(alias="Idea Title", default="")
    campaign_title: str = Field(alias="Idea Campaign", default="")
    question: str = Field(alias="Assessment Question", default="")
    email: str = Field(alias="Email", default="")
    date: str = Field(alias="Date", default="")

    @validator("email", pre=True)
    @classmethod
    def lower_email(cls, value):
        """Use lowercase only for emails."""
        return value.lower()

    class Config:
        """Extra configuration options."""

        anystr_strip_whitespace = True
