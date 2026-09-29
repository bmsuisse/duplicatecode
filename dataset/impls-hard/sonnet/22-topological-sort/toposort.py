"""Topological ordering of tasks with dependencies."""

from collections import deque
from collections.abc import Hashable, Iterable, Mapping


class CycleError(ValueError):
    """Raised when the dependency graph contains a cycle."""

    def __init__(self, nodes: list[Hashable]) -> None:
        super().__init__(f"dependency cycle detected involving: {nodes!r}")
        self.nodes = nodes


def topological_sort[T: Hashable](dependencies: Mapping[T, Iterable[T]]) -> list[T]:
    """Order tasks so that each comes after all of its prerequisites.

    ``dependencies`` maps a task to the tasks it depends on. Tasks that only
    appear as prerequisites are included too. The result is stable with respect
    to first appearance. Raises ``CycleError`` for circular dependencies.
    """
    prerequisites: dict[T, set[T]] = {}
    dependents: dict[T, list[T]] = {}
    for task, deps in dependencies.items():
        prerequisites.setdefault(task, set())
        dependents.setdefault(task, [])
        for dep in deps:
            prerequisites.setdefault(dep, set())
            dependents.setdefault(dep, [])
            if task not in dependents[dep]:
                dependents[dep].append(task)
            prerequisites[task].add(dep)

    ready = deque(task for task, deps in prerequisites.items() if not deps)
    order: list[T] = []
    while ready:
        task = ready.popleft()
        order.append(task)
        for follower in dependents[task]:
            prerequisites[follower].discard(task)
            if not prerequisites[follower]:
                ready.append(follower)

    if len(order) != len(prerequisites):
        raise CycleError([t for t, deps in prerequisites.items() if deps])
    return order
