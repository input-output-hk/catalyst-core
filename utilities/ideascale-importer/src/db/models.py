class Challenge:
    def __init__(
        self,
        id: int, election: int,
        type: str, title: str, description: str,
        rewards_currency: str, rewards_total: int,
    ):
        self.id = id
        self.election = election
        self.type = type
        self.title = title
        self.description = description
        self.rewards_currency = rewards_currency
        self.rewards_total = rewards_total
