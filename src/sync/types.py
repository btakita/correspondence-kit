from dataclasses import dataclass, field


@dataclass
class Message:
    id: str
    thread_id: str
    from_: str
    date: str
    subject: str
    body: str


@dataclass
class Thread:
    id: str
    label: str
    subject: str
    messages: list[Message] = field(default_factory=list)
    last_date: str = ""