import json
import logging
import os
from dataclasses import dataclass

logger = logging.getLogger(__name__)

@dataclass(frozen=True)
class User:
    wikidot_id: int
    created_at: int
    full_name: str
    slug: str
    account_type: str
    karma: int


def load_users(users_directory):
    users = []

    for filename in os.listdir(users_directory):
        if filename == "pending.json":
            logger.debug("Skipping pending.json")
            continue

        logger.info("Loading users from %s", filename)
        with open(os.path.join(users_directory, filename)) as file:
            users_data = json.load(file)

        for _, user_data in users_data:
            users.append(
                User(
                    full_name=user_data["full_name"],
                    slug=user_data["slug"],
                    created_at=user_data["wikidot_user_since"],
                    account_type=user_data["account_type"],
                    karma=user_data["activity"],
                    wikidot_id=user_data["user_id"],
                )
            )

    return users
