"""Retry a callable with exponential backoff."""

import random
import time
from collections.abc import Callable


def retry_with_backoff[T](
    func: Callable[[], T],
    *,
    max_attempts: int = 5,
    base_delay: float = 0.5,
    factor: float = 2.0,
    max_delay: float = 30.0,
    jitter: bool = True,
    retry_on: tuple[type[BaseException], ...] = (Exception,),
    sleep: Callable[[float], None] = time.sleep,
) -> T:
    """Call ``func`` until it succeeds, waiting longer after each failure.

    The delay before retry ``n`` (starting at 0) is ``base_delay * factor**n``,
    capped at ``max_delay``. With ``jitter`` the delay is randomised between 50%
    and 100% of that value. If every attempt fails, the last exception is raised.
    """
    if max_attempts < 1:
        raise ValueError("max_attempts must be at least 1")
    if base_delay < 0 or max_delay < 0 or factor < 1:
        raise ValueError("delays must be non-negative and factor must be >= 1")

    last_error: BaseException | None = None
    for attempt in range(max_attempts):
        try:
            return func()
        except retry_on as exc:
            last_error = exc
            if attempt == max_attempts - 1:
                break
            delay = min(max_delay, base_delay * factor**attempt)
            if jitter:
                delay *= random.uniform(0.5, 1.0)
            sleep(delay)
    assert last_error is not None
    raise last_error
