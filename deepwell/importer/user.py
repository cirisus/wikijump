import json
import glob
import os
from dataclasses import dataclass

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

    for path in glob.iglob(os.path.join(users_directory, "*.json")):
        if os.path.basename(path) == "pending.json":
            # Doesn't contain user info, skip
            continue

        print(f"Loading users from {path}")

        with open(path) as file:
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
