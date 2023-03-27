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
    wikitext: str
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


@dataclass(frozen=True)
class ForumCategory:
    wikidot_id: int
    title: str
    description: str
    posts: int
    threads: int
    last_user: Optional[int]
    full_scan: bool
    last_page: int


@dataclass(frozen=True)
class ForumThread:
    wikidot_id: int
    forum_category_id: int
    created_at: int
    user_id: Optional[int]
    title: str
    description: str
    posts: int
    last_post_time: Optional[int]
    last_post_user: Optional[int]
    locked: bool
    sticky: bool


@dataclass(frozen=True)
class ForumPost:
    wikidot_id: int
    forum_thread_id: int
    parent_post_id: Optional[int]
    title: str
    created_at: int
    user_id: int
    last_edit_at: Optional[int]
    last_edit_by: Optional[int]


@dataclass(frozen=True)
class ForumRevision:
    wikidot_id: int
    forum_post_id: int
    created_at: int
    user_id: int
    title: str
    html: str
    wikitext: Optional[str]
