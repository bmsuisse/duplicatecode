# Retry a callable with exponential backoff

- **Language:** python
- **Target file:** `retry_with_backoff.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def retry_with_backoff(fn: Callable[[], T], *, max_attempts: int = 5, base_delay: float = 0.1, factor: float = 2.0, max_delay: float = 5.0, retry_on: tuple[type[BaseException], ...] = (Exception,), sleep: Callable[[float], None] = time.sleep) -> T
```

## Behavior

Call `fn()` until it returns. If it raises an exception matching `retry_on`, sleep `min(max_delay, base_delay * factor**(attempt_index))` (attempt_index starts at 0 for the first failure) and try again. After `max_attempts` total calls, re-raise the last exception. No jitter. The injected `sleep` must be used for waiting.

## Edge cases

- max_attempts < 1 -> ValueError.
- Exceptions not in `retry_on` propagate immediately without sleeping.
- No sleep after the final failed attempt.
- Return value of first success is returned unchanged (including None).

## Examples

- fn fails twice then returns 7, base_delay=0.1, factor=2 -> returns 7; sleep called with 0.1 then 0.2.
- fn always raises KeyError, max_attempts=3, retry_on=(KeyError,) -> raises KeyError after 3 calls, sleeps [0.1, 0.2].
- base_delay=1, factor=10, max_delay=5, 3 failures then success -> sleeps [1, 5, 5].
- fn raises ValueError, retry_on=(KeyError,) -> ValueError raised, fn called once.
