def normalize_email(value: str) -> str | None:
    # Trim whitespace
    email = value.strip()

    # Check if empty
    if not email:
        return None

    # Check total length
    if len(email) > 254:
        return None

    # Check for exactly one @
    if email.count("@") != 1:
        return None

    local, domain = email.split("@")

    # Validate local part
    if not local or len(local) > 64:
        return None

    # Local part must not start or end with '.'
    if local.startswith(".") or local.endswith("."):
        return None

    # Check for '..' in local part
    if ".." in local:
        return None

    # Check valid characters in local part
    valid_local_chars = set(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.!#$%&'*+/=?^_`{|}~-"
    )
    for char in local:
        if char not in valid_local_chars:
            return None

    # Validate domain
    if not domain:
        return None

    domain_lower = domain.lower()

    # Domain must have at least one dot
    if "." not in domain_lower:
        return None

    # Split domain into labels
    labels = domain_lower.split(".")

    # Check each label
    for i, label in enumerate(labels):
        if not label or len(label) > 63:
            return None

        # Label must not start or end with '-'
        if label.startswith("-") or label.endswith("-"):
            return None

        # Check valid characters in label
        for char in label:
            if not (char.isalnum() or char == "-"):
                return None

    # Last label must be at least 2 letters
    if len(labels[-1]) < 2 or not labels[-1].isalpha():
        return None

    return local + "@" + domain_lower
