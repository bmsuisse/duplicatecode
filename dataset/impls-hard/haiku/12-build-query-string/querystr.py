from urllib.parse import urlencode


def build_query_string(params: dict[str, str | list[str]]) -> str:
    """Build URL query string from dictionary."""
    # Flatten lists into multiple parameters
    flat_params = []
    for key, value in params.items():
        if isinstance(value, list):
            for v in value:
                flat_params.append((key, v))
        else:
            flat_params.append((key, value))

    return urlencode(flat_params)
