from typing import Any


def flatten_tree(node: dict[str, Any], children_key: str = "children") -> list[dict[str, Any]]:
    """Flatten tree structure to flat list using depth-first traversal."""
    result = []

    def traverse(current: dict[str, Any]) -> None:
        # Add current node without children
        result.append({k: v for k, v in current.items() if k != children_key})
        # Recursively process children
        if children_key in current:
            for child in current[children_key]:
                traverse(child)

    traverse(node)
    return result
