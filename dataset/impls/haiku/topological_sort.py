from collections.abc import Iterable, Mapping


def topo_sort(deps: Mapping[str, Iterable[str]]) -> list[str]:
    # Build the graph
    all_nodes = set(deps.keys())
    in_degree: dict[str, int] = {}
    adj_list: dict[str, set[str]] = {}

    # Initialize in_degree and adj_list for all nodes
    for node in all_nodes:
        in_degree[node] = 0
        adj_list[node] = set()

    # Add dependencies as reverse edges
    for node, dependencies in deps.items():
        dep_list = list(dependencies)
        for dep in dep_list:
            all_nodes.add(dep)
            if dep not in in_degree:
                in_degree[dep] = 0
                adj_list[dep] = set()
            adj_list[dep].add(node)
            in_degree[node] += 1

    # Initialize: nodes with no incoming edges (but need to count properly)
    # Recalculate in_degree to account for implicit nodes
    in_degree_final: dict[str, int] = {node: 0 for node in all_nodes}

    for node, dependencies in deps.items():
        dep_set = set(dependencies)
        in_degree_final[node] = len(dep_set)

    # Kahn's algorithm with alphabetical ordering
    queue = sorted([node for node in all_nodes if in_degree_final[node] == 0])
    result = []

    while queue:
        # Pick alphabetically smallest
        current = queue.pop(0)
        result.append(current)

        # Process neighbors
        neighbors = adj_list.get(current, set())
        for neighbor in sorted(neighbors):
            in_degree_final[neighbor] -= 1
            if in_degree_final[neighbor] == 0:
                queue.append(neighbor)
                queue.sort()

    # Check for cycle
    if len(result) != len(all_nodes):
        raise ValueError("cycle detected")

    return result
