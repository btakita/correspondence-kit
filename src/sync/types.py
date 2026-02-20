import msgspec


class Message(msgspec.Struct):
    id: str
    thread_id: str
    from_: str
    date: str
    subject: str
    body: str


class Thread(msgspec.Struct):
    id: str
    label: str
    subject: str
    messages: list[Message] = []
    last_date: str = ""


class LabelState(msgspec.Struct):
    uidvalidity: int
    last_uid: int


class AccountSyncState(msgspec.Struct):
    labels: dict[str, LabelState] = {}


class SyncState(msgspec.Struct):
    accounts: dict[str, AccountSyncState] = {}
    labels: dict[str, LabelState] = {}  # legacy flat format, for migration


def load_state(data: bytes) -> SyncState:
    """Decode sync state, migrating legacy flat format if needed."""
    state = msgspec.json.decode(data, type=SyncState)
    # Migrate: if old flat labels exist and accounts is empty, move to _legacy
    if state.labels and not state.accounts:
        state.accounts["_legacy"] = AccountSyncState(labels=state.labels)
        state.labels = {}
    return state
