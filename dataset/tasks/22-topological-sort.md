# Topological sort of dependency graph

- **Language:** python
- **Target file:** `topological_sort.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def topo_sort(deps: Mapping[str, Iterable[str]]) -> list[str]
```

## Behavior

`deps` maps node -> nodes it depends on. Return an order where every node appears after its dependencies. Nodes appearing only as dependencies are included. Deterministic tie-break: among ready nodes, choose alphabetically smallest first (Kahn's algorithm with sorted ready set). Cycle -> raise ValueError with message starting 'cycle detected'.

## Edge cases

- Empty -> [].
- Self dependency -> cycle.
- Duplicate dependencies are tolerated.

## Examples

- {"app":["lib","log"],"lib":["log"]} -> ["log","lib","app"]
- {"b":[],"a":[]} -> ["a","b"]
- {"a":["b"],"b":["a"]} -> ValueError
- {"a":["x"]} -> ["x","a"]
