import re
import unicodedata


def slugify(text: str, *, max_length: int | None = None, separator: str = "-") -> str:
    # Lowercase
    text = text.lower()

    # Custom replacements
    text = text.replace("ß", "ss").replace("æ", "ae").replace("ø", "o")

    # Transliterate using NFKD
    text = unicodedata.normalize("NFKD", text)
    text = "".join(c for c in text if not unicodedata.combining(c))

    # Replace runs of non-alphanumeric characters with separator
    text = re.sub(r"[^a-z0-9]+", separator, text)

    # Strip leading/trailing separators
    text = text.strip(separator)

    # Truncate if needed
    if max_length is not None:
        text = text[:max_length]
        text = text.rstrip(separator)

    return text
