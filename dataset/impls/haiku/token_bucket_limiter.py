import time
from collections.abc import Callable


class TokenBucket:
    def __init__(
        self,
        capacity: float,
        refill_per_second: float,
        clock: Callable[[], float] = time.monotonic,
    ) -> None:
        if capacity <= 0:
            raise ValueError("capacity must be > 0")
        if refill_per_second <= 0:
            raise ValueError("refill_per_second must be > 0")

        self.capacity = capacity
        self.refill_per_second = refill_per_second
        self.clock = clock
        self.tokens = capacity
        self.last_refill_time = clock()

    def _refill(self) -> None:
        now = self.clock()
        elapsed = max(0, now - self.last_refill_time)
        new_tokens = elapsed * self.refill_per_second
        self.tokens = min(self.capacity, self.tokens + new_tokens)
        self.last_refill_time = now

    def try_acquire(self, tokens: float = 1.0) -> bool:
        self._refill()
        if self.tokens >= tokens:
            self.tokens -= tokens
            return True
        return False

    def available(self) -> float:
        self._refill()
        return self.tokens

    def wait_time(self, tokens: float = 1.0) -> float:
        if tokens > self.capacity:
            raise ValueError("tokens exceed capacity")

        self._refill()
        if self.tokens >= tokens:
            return 0.0

        needed = tokens - self.tokens
        return needed / self.refill_per_second
