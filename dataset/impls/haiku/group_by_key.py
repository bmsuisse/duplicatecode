from collections.abc import Callable, Iterable


def group_by[T, K: (str | int)](items: Iterable[T], key: Callable[[T], K]) -> dict[K, list[T]]:
    result: dict[K, list[T]] = {}
    for item in items:
        k = key(item)
        if k not in result:
            result[k] = []
        result[k].append(item)
    return result


def count_by[T, K: (str | int)](items: Iterable[T], key: Callable[[T], K]) -> dict[K, int]:
    result: dict[K, int] = {}
    for item in items:
        k = key(item)
        result[k] = result.get(k, 0) + 1
    return result
