from dataclasses import dataclass
from typing import List, Optional


@dataclass(frozen=True)
class User:
    wikidot_id: int
    created_at: int
    full_name: str
    slug: str
    account_type: str
    karma: int


@dataclass(frozen=True)
class Page:
    wikidot_id: int
    title: str
    slug: str
    tags: List[str]
    locked: bool
    discussion_thread_id: Optional[int]


@dataclass(frozen=True)
class PageRevision:
    wikidot_id: int
    revision_number: int
    created_at: int
    flags: str
    page_id: int
    user_id: int
    comments: str


@dataclass(frozen=True)
class PageVote:
    page_id: int
    user_id: int
    value: int


@dataclass(frozen=True)
class File:
    wikidot_id: int
    page_id: int
    user_id: int
    created_at: int
    name: str
    url: str
    size: int
    mime: str
    mime_description: str
    internal_version: int
