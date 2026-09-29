from typing import Any


def flatten_tree(node: Any, child_key: str = "children") -> list[Any]:
    """Flatten a tree of nodes into a flat list."""
    result = [node]
    children = node.get(child_key, []) if isinstance(node, dict) else []
    for child in children:
        result.extend(flatten_tree(child, child_key))
    return result
