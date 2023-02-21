import json

from .structures import User

def load_users(user_path, models=None):
    with open(user_path) as file:
        data = json.load(file)

    if models is None:
        models = []

    for _, user in data:
        models.append(User(
            wikidot_id=user_data["wikidot_id"],
            created_at=user_data["created_at"],
            full_name=user_data["full_name"],
            slug=user_data["slug"],
            account_type=user_data["account_type"],
            karma=user_data["karma"],
        ))
    return models
