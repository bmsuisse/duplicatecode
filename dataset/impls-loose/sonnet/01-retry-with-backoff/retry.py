"""Retry helper with exponential backoff."""

import random
import time
from collections.abc import Callable


def retry[T](
    func: Callable[[], T],
    *,
    attempts: int = 5,
    base_delay: float = 0.5,
    factor: float = 2.0,
    max_delay: float = 30.0,
    jitter: bool = False,
    exceptions: tuple[type[BaseException], ...] = (Exception,),
    sleep: Callable[[float], None] = time.sleep,
) -> T:
    """Call ``func`` until it succeeds, waiting exponentially longer between tries.

    Raises the last error once ``attempts`` calls have failed.
    """
    if attempts < 1:
        raise ValueError("attempts must be at least 1")
    delay = base_delay
    for attempt in range(1, attempts + 1):
        try:
            return func()
        except exceptions:
            if attempt == attempts:
                raise
        wait = min(delay, max_delay)
        sleep(random.uniform(0, wait) if jitter else wait)
        delay *= factor
    raise AssertionError("unreachable")
