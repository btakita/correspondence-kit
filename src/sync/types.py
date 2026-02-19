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


class SyncState(msgspec.Struct):
    labels: dict[str, LabelState] = {}
