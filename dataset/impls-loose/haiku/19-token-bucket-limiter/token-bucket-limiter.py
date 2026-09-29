import time


class TokenBucketLimiter:
    """Rate limiter using token bucket algorithm."""

    def __init__(self, capacity: float, refill_rate: float):
        self.capacity = capacity
        self.refill_rate = refill_rate
        self.tokens = capacity
        self.last_refill = time.time()

    def allow(self, tokens_needed: float = 1.0) -> bool:
        """Check if the action is allowed and consume tokens if so."""
        now = time.time()
        elapsed = now - self.last_refill
        self.tokens = min(self.capacity, self.tokens + elapsed * self.refill_rate)
        self.last_refill = now

        if self.tokens >= tokens_needed:
            self.tokens -= tokens_needed
            return True
        return False
