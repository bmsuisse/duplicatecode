import re
import unicodedata


def slugify(text: str) -> str:
    """Convert text to a URL slug.

    Converts to lowercase, removes accents, and replaces spaces/special chars with hyphens.
    """
    # Normalize unicode characters (decompose accented chars)
    text = unicodedata.normalize("NFKD", text)
    # Remove non-ASCII characters
    text = text.encode("ascii", "ignore").decode("ascii")
    # Convert to lowercase
    text = text.lower()
    # Replace spaces and underscores with hyphens
    text = re.sub(r"[\s_]+", "-", text)
    # Remove any character that's not alphanumeric or hyphen
    text = re.sub(r"[^a-z0-9\-]", "", text)
    # Remove multiple consecutive hyphens
    text = re.sub(r"-+", "-", text)
    # Strip hyphens from start and end
    text = text.strip("-")
    return text
