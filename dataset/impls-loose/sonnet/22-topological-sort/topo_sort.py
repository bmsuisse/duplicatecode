"""Topological ordering of tasks with dependencies."""

from collections import deque
from collections.abc import Hashable, Iterable, Mapping


class CycleError(ValueError):
    """Raised when the dependency graph contains a cycle."""

    def __init__(self, nodes: list[Hashable]) -> None:
        super().__init__(f"dependency cycle involving: {nodes!r}")
        self.nodes = nodes


def topological_sort[T: Hashable](dependencies: Mapping[T, Iterable[T]]) -> list[T]:
    """Order tasks so each comes after everything it depends on.

    ``dependencies`` maps a task to the tasks it depends on. Tasks that appear only
    as dependencies are included. Ties keep input order (Kahn's algorithm).
    """
    deps: dict[T, set[T]] = {}
    for task, requires in dependencies.items():
        deps.setdefault(task, set()).update(requires)
        for req in requires:
            deps.setdefault(req, set())
    dependents: dict[T, list[T]] = {task: [] for task in deps}
    remaining = {task: len(reqs) for task, reqs in deps.items()}
    for task, reqs in deps.items():
        for req in reqs:
            dependents[req].append(task)
    ready = deque(task for task, count in remaining.items() if count == 0)
    order: list[T] = []
    while ready:
        task = ready.popleft()
        order.append(task)
        for dependent in dependents[task]:
            remaining[dependent] -= 1
            if remaining[dependent] == 0:
                ready.append(dependent)
    if len(order) != len(deps):
        raise CycleError([task for task, count in remaining.items() if count > 0])
    return order
