from dataclasses import dataclass
from typing import Generic, TypeVar


K = TypeVar("K")
V = TypeVar("V")


@dataclass
class Pair(Generic[K, V]):
    key: K
    value: V
