"""Flatten a tree of nodes into a list."""

from collections.abc import Iterator, Mapping
from typing import Any


def iter_tree(
    root: Mapping[str, Any] | list[Mapping[str, Any]], children_key: str = "children"
) -> Iterator[Mapping[str, Any]]:
    """Yield nodes depth-first, parents before children (pre-order).

    ``root`` may be a single node or a list of top-level nodes. Uses an explicit
    stack so very deep trees don't hit the recursion limit.
    """
    roots = [root] if isinstance(root, Mapping) else list(root)
    stack = list(reversed(roots))
    while stack:
        node = stack.pop()
        yield node
        stack.extend(reversed(node.get(children_key) or []))


def flatten_tree(
    root: Mapping[str, Any] | list[Mapping[str, Any]], children_key: str = "children"
) -> list[dict[str, Any]]:
    """Return a flat list of shallow node copies without their children key."""
    return [
        {k: v for k, v in node.items() if k != children_key}
        for node in iter_tree(root, children_key)
    ]
