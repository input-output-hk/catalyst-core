"""Models for reviews."""
from __future__ import annotations
from pydantic import BaseModel, HttpUrl, Field, field_validator, model_validator
from typing import List, Optional
from enum import IntEnum
import re
import json


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

    @field_validator("anon_id", mode="before")
    @classmethod
    def parse_anon_id(cls, value):
        """Get anonymized id from a list. The first one is the main that is used."""
        id = value.split(",")[0]
        return id

    @field_validator("email", mode="before")
    @classmethod
    def lower_email(cls, value):
        """Use lowercase only for emails."""
        return value.lower()

    @model_validator(mode="before")
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

class LightPa(Model):
    pa_anon_id: int = Field(default=0)
    pa_email: str = Field(default="")
    pa_level: int = Field(default=0)
    reviews_count: int = Field(default=0)
    allocated_reviews_count: int = Field(default=0)
    unallocated_reviews_count: int = Field(default=0)

class Author(Model):
    """Represents an author."""

    id: int = Field(default=0)
    name: str = Field(default="")
    email: str = Field(default="")
    user_name: str = Field(default="")


class Challenge(Model):
    """Represents a challenge."""

    id: int = Field(default=0)
    title: str = Field(default="")
    funds: Optional[int] = Field(default=None)


class Proposal(Model):
    """Represents a proposal."""

    id: int = Field(default=0)
    url: HttpUrl
    title: str = Field(default="")
    challenge_id: int = Field(alias="campaign_id", default=0)
    times_picked: int = Field(default=0)
    funds: Optional[int] = Field(default=None)
    allocations: List[Allocation] = Field(default=[])
    authors: List[Author] = Field(default=[])
    public_email: Optional[str] = Field(default=None)
    stage_id: Optional[int] = Field(default=None)
    extra: Optional[dict] = Field(default=None)
    review_stats: Optional[dict] = Field(default=None)
    archived: Optional[bool] = Field(default=False)

    @model_validator(mode="before")
    @classmethod
    def assign_authors_if_any(cls, values):
        """Assign proposers/co-proposers merging different ideascale fields."""
        authors = []
        if "stage_label" in values:
            if values["stage_label"] == "Archive":
                values["archived"] = True
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

class StatsProposal(Model):
    id: int = Field(default=0)
    url: HttpUrl
    title: str = Field(default="")
    challenge_id: int = Field(default=0)
    no_reviews: int = Field(default=0)
    avg_score: float = Field(default=0)


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

    id: int = Field(default=0)
    assessor: str = Field(alias="Assessor", default="")
    impact_note: str = Field(alias="Impact / Alignment Note", default="")
    impact_rating: int = Field(alias="Impact / Alignment Rating", default=0)
    feasibility_note: str = Field(alias="Feasibility Note", default="")
    feasibility_rating: int = Field(alias="Feasibility Rating", default=0)
    auditability_note: str = Field(alias="Auditability Note", default="")
    auditability_rating: int = Field(alias="Auditability Rating", default=0)
    classification: Optional[str] = Field(alias="Result", default=None)
    level: Optional[int] = Field(default=None)
    allocated: Optional[bool] = Field(default=None)
    pa: Optional[Pa] = Field(default=None)
    proposal: Optional[Proposal] = Field(default=None)
    fund: Optional[int] = Field(default=None)
    proposal_id: Optional[int] = Field(default=None)

    @property
    def full_note(self):
        """Return the full not combining the single criteria notes."""
        return " ".join([self.impact_note, self.feasibility_note, self.auditability_note])
    
    def valid_by_length(self, min_length):
        return (len(self.impact_note) >= min_length and len(self.feasibility_note) >= min_length and len(self.auditability_note) >= min_length)
    
    @model_validator(mode="before")
    @classmethod
    def adjust_id(cls, values):
        """Set and id based on fund_id if present."""
        if "fund" in values:
            values["id"] = int(values["fund"]) * 100000 + int(values["id"])
        return values

def invert_pair(values):
    left = values["left"]
    left_criterium = values["left_criterium"]
    values["left"] = values["right"]
    values["left_criterium"] = values["right_criterium"]
    values["right"] = left
    values["right_criterium"] = left_criterium
    return values


class SimilarPair(Model):
    """Represent a pair of assessments similar."""

    left: Review
    right: Review
    score: float
    left_criterium: str
    right_criterium: str

    @model_validator(mode="before")
    @classmethod
    def lower_always_left(cls, values):
        """Force the lower score to be on `left` field."""
        if values["left"].fund < values["right"].fund:
            values = invert_pair(values)
        elif values["left"].id > values["right"].id:
            values = invert_pair(values)
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

    @model_validator(mode="before")
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
    
    @field_validator("email", mode="before")
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

    @field_validator("score", mode="before")
    @classmethod
    def catch_na(cls, value):
        """Catch NA."""
        if value == "NA":
            return 1
        return value

    @field_validator("email", mode="before")
    @classmethod
    def lower_email(cls, value):
        """Use lowercase only for emails."""
        return value.lower()

    @field_validator("question", mode="before")
    @classmethod
    def tranform_question(cls, value):
        return value.replace('\n', '').replace('\r', '').replace(' ', '')

    class Config:
        """Extra configuration options."""

        anystr_strip_whitespace = True


class IdeascaleExportedReviewResult(Model):
    """Represents a review exported from Ideascale Excel file."""

    idea_id: int = Field(alias="Idea ID", default=0)
    idea_title: str = Field(alias="Title", default="")
    campaign_title: str = Field(alias="Idea Campaign", default="")
    question: str = Field(alias="Assessment Question", default="")
    email: str = Field(alias="Email", default="")
    date: str = Field(alias="Date", default="")

    @field_validator("email", mode="before")
    @classmethod
    def lower_email(cls, value):
        """Use lowercase only for emails."""
        return value.lower()

    @field_validator("question", mode="before")
    @classmethod
    def tranform_question(cls, value):
        return value.replace('\n', '').replace('\r', '').replace(' ', '')

    class Config:
        """Extra configuration options."""

        anystr_strip_whitespace = True

class LightSimilarPair(Model):
    """Represent a pair of assessments similar with light information."""

    left: int
    right: int
    score: float
    left_criterium: str
    right_criterium: str

class LightAiFlag(Model):
    """Represent a review criteria ai flag with light information."""

    review: int
    avg_generated_prob: float
    generated_prob: float

class LightProfanity(Model):
    """Represent a review criteria profanity flag with light information."""

    review: int = Field(alias="review_id")
    score: float
    criterium: str

class FlagType(IntEnum):
    """Enum to describe possible flag types."""

    profanity = 0
    similarity = 1
    ai_generated = 2


class Flag(Model):
    """Submodel to store reviews flags."""

    flag_type: FlagType
    score: float
    related_reviews: List[int] = Field([])
    related_criteria: List[str] = Field([])

class ReviewWithFlags(Review):
    """Represent a review with flags."""

    flags: List[Flag] | None

class Moderator(Model):
    """Represents a Moderator."""

    row_id: Optional[int]
    email: Optional[str]
    name: Optional[str]
    id: Optional[str] = Field(alias="ids")
    encrypted_email: Optional[str]
    password: Optional[str]
    salt: Optional[str]
    hashed_password: Optional[str]
    hashed_email: Optional[str]
    allocations: Optional[ReviewWithFlags] = Field(default=[])

    @field_validator("email", mode="before")
    @classmethod
    def lower_email(cls, value):
        """Use lowercase only for emails."""
        return value.lower()

class ModeratorSQL(Model):
    """Represents a Moderator in SQL."""

    row_id: int
    id: str = Field(alias="hashed_email")
    id2: str = Field(alias="hashed_password")
    id3: str = Field(alias="salt")
    value: str = Field(alias="extra")

    @model_validator(mode="before")
    @classmethod
    def parse_value(cls, values):
        extra = {
            "active": True,
            "force_reset": True,
            "role": 2,
            "encrypted_email": values["encrypted_email"],
        }
        values["extra"] = json.dumps(extra)
        return values

class ModeratorAllocationSQL(Model):
    """Represents a Moderator allocation in SQL."""

    review_id: int
    user_id: int
