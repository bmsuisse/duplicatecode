"""Retry a callable with exponential backoff."""

import time
from collections.abc import Callable


def retry_with_backoff[T](
    fn: Callable[[], T],
    *,
    max_attempts: int = 5,
    base_delay: float = 0.1,
    factor: float = 2.0,
    max_delay: float = 5.0,
    retry_on: tuple[type[BaseException], ...] = (Exception,),
    sleep: Callable[[float], None] = time.sleep,
) -> T:
    """Call ``fn`` until it succeeds, sleeping exponentially between failures."""
    if max_attempts < 1:
        raise ValueError("max_attempts must be at least 1")

    attempt = 0
    while True:
        try:
            return fn()
        except retry_on:
            if attempt + 1 >= max_attempts:
                raise
            sleep(min(max_delay, base_delay * factor**attempt))
            attempt += 1
