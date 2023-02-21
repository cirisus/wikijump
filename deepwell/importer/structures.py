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
    discussion_thread_id: Optional[int]


@dataclass(frozen=True)
class PageRevision:
    wikidot_id: Optional[int]
    revision_number: int
    created_at: datetime
    flags: str
    page_id: int
    site_id: int
    user_id: int
    wikitext: str
    html: str
    slug: str
    title: str
    tags: List[str]
    comments: str


@dataclass(frozen=True)
class PageVote:
    page_id: int
    user_id: int
    value: int


@dataclass(frozen=True)
class File:
    wikidot_id: Optional[int]
    page_id: int
    name: str
    mime: str
    size: int
    user_id: int
    created_at: datetime
