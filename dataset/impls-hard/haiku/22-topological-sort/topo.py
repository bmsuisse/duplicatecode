def topological_sort(graph: dict[str, list[str]]) -> list[str] | None:
    """Topological sort detecting circular dependencies."""
    in_degree = {node: 0 for node in graph}
    for node, neighbors in graph.items():
        for neighbor in neighbors:
            in_degree[neighbor] = in_degree.get(neighbor, 0) + 1

    queue = [node for node in graph if in_degree[node] == 0]
    result = []

    while queue:
        node = queue.pop(0)
        result.append(node)

        for neighbor in graph.get(node, []):
            in_degree[neighbor] -= 1
            if in_degree[neighbor] == 0:
                queue.append(neighbor)

    # Check for cycles
    if len(result) != len(graph):
        return None

    return result
