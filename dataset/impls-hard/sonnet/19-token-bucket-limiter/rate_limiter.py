"""Token bucket rate limiter."""

import threading
import time
from collections.abc import Callable


class TokenBucket:
    """Allows ``rate`` operations per second on average with bursts up to ``capacity``."""

    def __init__(
        self,
        rate: float,
        capacity: float,
        clock: Callable[[], float] = time.monotonic,
    ) -> None:
        if rate <= 0 or capacity <= 0:
            raise ValueError("rate and capacity must be positive")
        self._rate = rate
        self._capacity = capacity
        self._clock = clock
        self._tokens = capacity
        self._updated = clock()
        self._lock = threading.Lock()

    def _refill(self) -> None:
        now = self._clock()
        elapsed = max(0.0, now - self._updated)
        self._tokens = min(self._capacity, self._tokens + elapsed * self._rate)
        self._updated = now

    def try_acquire(self, tokens: float = 1.0) -> bool:
        """Consume ``tokens`` and return True if available, otherwise return False."""
        if tokens <= 0 or tokens > self._capacity:
            raise ValueError("tokens must be positive and not exceed capacity")
        with self._lock:
            self._refill()
            if self._tokens >= tokens:
                self._tokens -= tokens
                return True
            return False

    def wait_time(self, tokens: float = 1.0) -> float:
        """Return the seconds until ``tokens`` become available (0 if now)."""
        with self._lock:
            self._refill()
            missing = tokens - self._tokens
            return max(0.0, missing / self._rate)

    def acquire(self, tokens: float = 1.0, sleep: Callable[[float], None] = time.sleep) -> None:
        """Block until ``tokens`` could be consumed."""
        while not self.try_acquire(tokens):
            sleep(self.wait_time(tokens))
