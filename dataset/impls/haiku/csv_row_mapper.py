from collections.abc import Callable, Iterable, Mapping, Sequence
from typing import Any


def map_rows(
    rows: Iterable[Sequence[str]],
    mapping: Mapping[str, tuple[str, Callable[[str], Any]]],
    *,
    has_header: bool = True,
) -> tuple[list[dict[str, Any]], list[str]]:
    records: list[dict[str, Any]] = []
    errors: list[str] = []

    rows_list = list(rows)
    if not rows_list:
        return ([], [])

    # Get header
    if has_header:
        header_row = rows_list[0]
        data_rows = rows_list[1:]
    else:
        header_row = [str(i) for i in range(len(rows_list[0]))]
        data_rows = rows_list

    # Build column index map (case-insensitive matching)
    col_index_map: dict[str, int] = {}

    for mapping_key, (output_key, converter) in mapping.items():
        mapping_key_lower = mapping_key.lower()
        if mapping_key_lower in [h.strip().lower() for h in header_row]:
            for i, header_cell in enumerate(header_row):
                if header_cell.strip().lower() == mapping_key_lower:
                    col_index_map[mapping_key_lower] = i
                    break
        else:
            if not has_header:
                # For no header, mapping keys are 0-based column indexes as strings
                if mapping_key in [str(i) for i in range(len(header_row))]:
                    col_index_map[mapping_key] = int(mapping_key)
                else:
                    raise ValueError(f"Missing header column in mapping: {mapping_key}")
            else:
                raise ValueError(f"Missing header column in mapping: {mapping_key}")

    # Process data rows
    for row_num, row in enumerate(data_rows, start=1):
        # Check if row is fully empty
        if not any(cell.strip() for cell in row):
            continue

        record: dict[str, Any] = {}
        row_has_error = False

        for mapping_key, (output_key, converter) in mapping.items():
            mapping_key_lower = mapping_key.lower() if has_header else mapping_key

            if mapping_key_lower not in col_index_map:
                continue

            col_idx = col_index_map[mapping_key_lower]

            # Get cell value
            if col_idx < len(row):
                cell = row[col_idx].strip()
            else:
                cell = ""

            # Handle blank cells
            if not cell:
                record[output_key] = None
            else:
                try:
                    record[output_key] = converter(cell)
                except ValueError as e:
                    errors.append(f"row {row_num}: {output_key}: {e!s}")
                    row_has_error = True
                    break

        if not row_has_error:
            records.append(record)

    return (records, errors)
