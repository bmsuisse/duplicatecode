from collections.abc import Mapping, Sequence
from typing import Any, Literal


def merge_records(
    records: Sequence[Mapping[str, Any]],
    rules: Mapping[str, Literal["most_recent", "most_complete", "first_non_null", "longest"]],
    *,
    updated_key: str = "updated_at",
) -> dict[str, Any]:
    if not records:
        raise ValueError("records cannot be empty")

    # Get union of all keys except updated_key
    all_keys = set()
    for record in records:
        all_keys.update(k for k in record if k != updated_key)

    # Helper to check if value is empty
    def is_empty(value: Any) -> bool:
        return value is None or value == ""

    # Get the max updated value - filter out None values first
    non_none_updates = [v for record in records if (v := record.get(updated_key)) is not None]
    max_updated: Any = max(non_none_updates) if non_none_updates else None

    result: dict[str, Any] = {updated_key: max_updated}

    for key in all_keys:
        rule = rules.get(key, "first_non_null")

        values_with_indices = [(i, record.get(key)) for i, record in enumerate(records)]

        if rule == "first_non_null":
            value = None
            for _, v in values_with_indices:
                if not is_empty(v):
                    value = v
                    break
            result[key] = value

        elif rule == "most_recent":
            value = None
            max_updated_record_idx = -1
            for i, record in enumerate(records):
                v = record.get(key)
                if not is_empty(v):
                    record_updated = record.get(updated_key)
                    if max_updated_record_idx == -1:
                        max_updated_record_idx = i
                        value = v
                    else:
                        best_updated = records[max_updated_record_idx].get(updated_key)
                        if (
                            record_updated is not None
                            and best_updated is not None
                            and record_updated >= best_updated
                        ):
                            max_updated_record_idx = i
                            value = v
            result[key] = value

        elif rule == "longest":
            value = None
            max_len = -1
            for _, v in values_with_indices:
                if not is_empty(v):
                    v_len = len(str(v))
                    if v_len > max_len:
                        max_len = v_len
                        value = v
            result[key] = value

        elif rule == "most_complete":
            # Find record with most non-empty fields
            best_record_idx = -1
            best_count = -1
            best_updated = None
            for i, record in enumerate(records):
                non_empty_count = sum(
                    1 for k, v in record.items() if k != updated_key and not is_empty(v)
                )
                if non_empty_count > best_count or (
                    non_empty_count == best_count
                    and best_updated is not None
                    and (
                        record.get(updated_key) is not None
                        and record.get(updated_key) > best_updated
                    )
                ):
                    best_record_idx = i
                    best_count = non_empty_count
                    best_updated = record.get(updated_key)

            if best_record_idx != -1:
                v = records[best_record_idx].get(key)
                if not is_empty(v):
                    result[key] = v
                else:
                    # Fall back to first_non_null
                    value = None
                    for _, vv in values_with_indices:
                        if not is_empty(vv):
                            value = vv
                            break
                    result[key] = value
            else:
                result[key] = None

    return result
