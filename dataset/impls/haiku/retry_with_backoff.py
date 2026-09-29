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
    if max_attempts < 1:
        raise ValueError("max_attempts must be >= 1")

    last_exception: BaseException | None = None
    for attempt in range(max_attempts):
        try:
            return fn()
        except BaseException as e:
            if not isinstance(e, retry_on):
                raise
            last_exception = e
            if attempt < max_attempts - 1:
                delay = min(max_delay, base_delay * (factor**attempt))
                sleep(delay)

    assert last_exception is not None
    raise last_exception
