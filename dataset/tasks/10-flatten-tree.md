# Flatten a tree of nodes

- **Language:** python
- **Target file:** `flatten_tree.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def flatten_tree(nodes: list[dict[str, Any]], *, children_key: str = "children", id_key: str = "id") -> list[dict[str, Any]]
```

## Behavior

Depth-first pre-order flatten. Each output item is a shallow copy of the node without its `children_key`, plus `depth` (roots 0) and `parent_id` (the parent's `id_key` value, None for roots) and `path` (list of ids from root to this node inclusive). Missing or None children key means leaf. Do not mutate the input.

## Edge cases

- Empty list -> [].
- Sibling order is preserved.
- Node lacking id_key -> KeyError.

## Examples

- [{"id":1,"children":[{"id":2},{"id":3,"children":[{"id":4}]}]}] -> [{id:1,depth:0,parent_id:None,path:[1]},{id:2,depth:1,parent_id:1,path:[1,2]},{id:3,depth:1,parent_id:1,path:[1,3]},{id:4,depth:2,parent_id:3,path:[1,3,4]}]
- [] -> []
- [{"id":"a","name":"x","children":None}] -> [{id:"a",name:"x",depth:0,parent_id:None,path:["a"]}]
