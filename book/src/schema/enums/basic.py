from dataclasses import dataclass
from enum import Enum


@dataclass
class Sample(Enum):
    VARIANT1 = 1
    VARIANT2 = 2
