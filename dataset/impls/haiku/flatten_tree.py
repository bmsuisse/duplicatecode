from typing import Any


def flatten_tree(
    nodes: list[dict[str, Any]],
    *,
    children_key: str = "children",
    id_key: str = "id",
) -> list[dict[str, Any]]:
    result = []

    def dfs(node: dict[str, Any], depth: int, parent_id: Any, path: list[Any]) -> None:
        # Get the node's ID
        node_id = node[id_key]

        # Create shallow copy without children_key
        output_node = {k: v for k, v in node.items() if k != children_key}

        # Add metadata
        output_node["depth"] = depth
        output_node["parent_id"] = parent_id
        output_node["path"] = path + [node_id]

        result.append(output_node)

        # Process children
        children = node.get(children_key)
        if children:
            for child in children:
                dfs(child, depth + 1, node_id, output_node["path"])

    for node in nodes:
        dfs(node, 0, None, [])

    return result
