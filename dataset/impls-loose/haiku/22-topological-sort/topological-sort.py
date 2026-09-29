def topological_sort(graph: dict[str, list[str]]) -> list[str]:
    """Topological sort of dependency graph."""
    visited = set()
    visiting = set()
    result = []

    def visit(node: str) -> None:
        if node in visited:
            return
        if node in visiting:
            msg = f"Cycle detected at {node}"
            raise ValueError(msg)

        visiting.add(node)
        for neighbor in graph.get(node, []):
            visit(neighbor)
        visiting.remove(node)
        visited.add(node)
        result.append(node)

    for node in graph:
        visit(node)

    return result
