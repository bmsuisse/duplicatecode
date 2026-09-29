# Token bucket rate limiter

- **Language:** python
- **Target file:** `token_bucket_limiter.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
class TokenBucket:
    def __init__(self, capacity: float, refill_per_second: float, clock: Callable[[], float] = time.monotonic) -> None
    def try_acquire(self, tokens: float = 1.0) -> bool
    def available(self) -> float
    def wait_time(self, tokens: float = 1.0) -> float
```

## Behavior

Starts full. Tokens refill continuously at `refill_per_second` up to `capacity`, computed lazily from the injected clock. `try_acquire` consumes and returns True if enough tokens, else consumes nothing and returns False. `available` returns current tokens after refill. `wait_time` returns seconds until `tokens` would be available (0.0 if already).

## Edge cases

- capacity <= 0 or refill <= 0 -> ValueError.
- tokens > capacity: try_acquire False; wait_time raises ValueError.
- Clock going backwards is treated as no elapsed time.

## Examples

- capacity 5, refill 1/s, clock t=0: try_acquire(5) True; try_acquire() False; at t=2 available()==2.0
- after draining at t=0: wait_time(3) == 3.0
- at t=100 available() == 5.0 (capped)
- TokenBucket(0, 1) -> ValueError
