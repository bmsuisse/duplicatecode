import time
from collections.abc import Callable
from typing import TypeVar

T = TypeVar("T")


def retry_with_backoff[T](
    func: Callable[..., T],
    *args,
    max_attempts: int = 3,
    initial_delay: float = 1.0,
    backoff_factor: float = 2.0,
    **kwargs,
) -> T:
    """Execute a callable with exponential backoff retry logic."""
    last_exception: Exception | None = None

    for attempt in range(max_attempts):
        try:
            return func(*args, **kwargs)
        except TimeoutError as e:
            last_exception = e
            if attempt < max_attempts - 1:
                delay = initial_delay * (backoff_factor**attempt)
                time.sleep(delay)

    if last_exception is not None:
        raise last_exception
    raise RuntimeError("Retry failed without exception")
