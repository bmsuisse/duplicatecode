"""Flatten a nested tree into a pre-order list."""

from collections.abc import Callable, Iterator
from typing import Any


def flatten_tree[N](root: N | list[N], get_children: Callable[[N], list[N] | None]) -> list[N]:
    """Return all nodes in pre-order (parent before children, left to right).

    ``root`` may be a single node or a list of top-level nodes. The traversal is
    iterative, so very deep trees do not hit the recursion limit.
    """
    roots: list[N] = list(root) if isinstance(root, list) else [root]
    result: list[N] = []
    stack: list[N] = roots[::-1]
    while stack:
        node = stack.pop()
        result.append(node)
        children = get_children(node)
        if children:
            stack.extend(reversed(children))
    return result


def iter_dict_tree(
    node: dict[str, Any], children_key: str = "children"
) -> Iterator[dict[str, Any]]:
    """Yield dict-based nodes in pre-order, using ``children_key`` for children."""
    yield from flatten_tree(node, lambda n: n.get(children_key))
