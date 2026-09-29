from collections.abc import Mapping
from typing import Any, Literal
from urllib.parse import quote


def build_query_string(
    params: Mapping[str, Any],
    *,
    array_format: Literal["repeat", "comma", "brackets"] = "repeat",
) -> str:
    def encode(value: str) -> str:
        return quote(value, safe="")

    pairs = []
    for key, value in params.items():
        if value is None:
            continue

        if isinstance(value, bool):
            pairs.append(f"{encode(key)}={encode(str(value).lower())}")
        elif isinstance(value, (list, tuple)):
            if not value:
                continue
            if array_format == "repeat":
                for item in value:
                    pairs.append(f"{encode(key)}={encode(str(item))}")
            elif array_format == "comma":
                comma_separated = ",".join(encode(str(item)) for item in value)
                pairs.append(f"{encode(key)}={comma_separated}")
            elif array_format == "brackets":
                for item in value:
                    pairs.append(f"{encode(key)}[]={encode(str(item))}")
        else:
            pairs.append(f"{encode(key)}={encode(str(value))}")

    return "&".join(pairs)
