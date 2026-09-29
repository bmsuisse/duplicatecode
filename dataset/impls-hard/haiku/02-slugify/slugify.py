import re
import unicodedata


def slugify(text: str) -> str:
    """Convert text to a URL-friendly slug."""
    # Normalize to NFD form and remove combining marks (accents)
    normalized = unicodedata.normalize("NFD", text)
    normalized = "".join(c for c in normalized if unicodedata.category(c) != "Mn")

    # Convert to lowercase
    slug = normalized.lower()

    # Replace spaces and underscores with hyphens
    slug = re.sub(r"[_\s]+", "-", slug)

    # Remove non-alphanumeric characters except hyphens
    slug = re.sub(r"[^a-z0-9-]", "", slug)

    # Remove multiple consecutive hyphens
    slug = re.sub(r"-+", "-", slug)

    # Remove leading/trailing hyphens
    slug = slug.strip("-")

    return slug
