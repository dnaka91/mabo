from dataclasses import dataclass
from typing import Literal


# region snippet
@dataclass
class SampleVariant1:
    tag: Literal["Variant1"]
    f1: int
    f2: int


@dataclass
class SampleVariant2:
    tag: Literal["Variant2"]
    field1: int
    field2: bool


# endregion snippet


Sample = SampleVariant1 | SampleVariant2
