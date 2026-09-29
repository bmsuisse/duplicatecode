"""Token bucket rate limiter."""

import time
from collections.abc import Callable


class TokenBucket:
    """Bucket that starts full and refills continuously from an injected clock."""

    def __init__(
        self,
        capacity: float,
        refill_per_second: float,
        clock: Callable[[], float] = time.monotonic,
    ) -> None:
        if capacity <= 0 or refill_per_second <= 0:
            raise ValueError("capacity and refill_per_second must be positive")
        self._capacity = capacity
        self._rate = refill_per_second
        self._clock = clock
        self._tokens = float(capacity)
        self._last = clock()

    def _refill(self) -> None:
        now = self._clock()
        elapsed = max(0.0, now - self._last)
        self._last = max(self._last, now)
        self._tokens = min(self._capacity, self._tokens + elapsed * self._rate)

    def try_acquire(self, tokens: float = 1.0) -> bool:
        self._refill()
        if tokens > self._tokens:
            return False
        self._tokens -= tokens
        return True

    def available(self) -> float:
        self._refill()
        return self._tokens

    def wait_time(self, tokens: float = 1.0) -> float:
        if tokens > self._capacity:
            raise ValueError("tokens exceed bucket capacity")
        self._refill()
        return max(0.0, (tokens - self._tokens) / self._rate)
