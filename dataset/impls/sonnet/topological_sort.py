"""Topological sort of a dependency graph."""

import heapq
from collections.abc import Iterable, Mapping


def topo_sort(deps: Mapping[str, Iterable[str]]) -> list[str]:
    """Order nodes so dependencies come first; ties broken alphabetically."""
    requires: dict[str, set[str]] = {}
    for node, needed in deps.items():
        requires.setdefault(node, set()).update(needed)
        for dep in needed:
            requires.setdefault(dep, set())
    dependents: dict[str, list[str]] = {node: [] for node in requires}
    for node, needed in requires.items():
        for dep in needed:
            dependents[dep].append(node)

    remaining = {node: len(needed) for node, needed in requires.items()}
    ready = [node for node, count in remaining.items() if count == 0]
    heapq.heapify(ready)
    order: list[str] = []
    while ready:
        node = heapq.heappop(ready)
        order.append(node)
        for child in dependents[node]:
            remaining[child] -= 1
            if remaining[child] == 0:
                heapq.heappush(ready, child)
    if len(order) != len(requires):
        stuck = sorted(node for node, count in remaining.items() if count > 0)
        raise ValueError(f"cycle detected among: {', '.join(stuck)}")
    return order
