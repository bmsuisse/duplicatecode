"""Flatten a tree of dict nodes depth-first."""

from typing import Any


def flatten_tree(
    nodes: list[dict[str, Any]], *, children_key: str = "children", id_key: str = "id"
) -> list[dict[str, Any]]:
    """Return nodes in pre-order with depth, parent_id and path attached."""
    result: list[dict[str, Any]] = []

    def visit(items: list[dict[str, Any]], parent_id: Any, path: list[Any]) -> None:
        for node in items:
            node_id = node[id_key]
            node_path = [*path, node_id]
            entry = {k: v for k, v in node.items() if k != children_key}
            entry.update(depth=len(path), parent_id=parent_id, path=node_path)
            result.append(entry)
            visit(node.get(children_key) or [], node_id, node_path)

    visit(nodes, None, [])
    return result
