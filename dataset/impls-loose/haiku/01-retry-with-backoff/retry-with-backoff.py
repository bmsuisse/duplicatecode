import time
from collections.abc import Callable


def retry_with_backoff[T](
    func: Callable[[], T],
    max_attempts: int = 3,
    base_delay: float = 1.0,
    backoff_factor: float = 2.0,
) -> T:
    """Retry a callable with exponential backoff.

    Args:
        func: The function to call
        max_attempts: Maximum number of attempts
        base_delay: Initial delay in seconds
        backoff_factor: Multiplier for delay between attempts

    Returns:
        The return value of the function

    Raises:
        The last exception if all attempts fail
    """
    last_error: BaseException | None = None
    for attempt in range(max_attempts):
        try:
            return func()
        except TimeoutError as e:
            last_error = e
            if attempt < max_attempts - 1:
                delay = base_delay * (backoff_factor**attempt)
                time.sleep(delay)
        except OSError as e:
            last_error = e
            if attempt < max_attempts - 1:
                delay = base_delay * (backoff_factor**attempt)
                time.sleep(delay)
        except RuntimeError as e:
            last_error = e
            if attempt < max_attempts - 1:
                delay = base_delay * (backoff_factor**attempt)
                time.sleep(delay)
    if last_error:
        raise last_error
    return func()
