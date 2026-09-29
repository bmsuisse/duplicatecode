# Flatten a node tree

- **Language:** typescript
- **Target file:** `flatten-tree-nodes.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export type FlatNode<T> = Omit<T, 'children'> & { depth: number; parentId: string | number | null; path: Array<string | number> }
export function flattenTree<T extends { id: string | number; children?: T[] | null }>(roots: readonly T[]): FlatNode<T>[]
```

## Behavior

Depth-first pre-order traversal. Each output item is a shallow copy of the node without `children`, plus depth (roots 0), parentId (null for roots) and path (ids root..self). null/undefined/missing children is a leaf. Does not mutate the input. Must not use recursion depth-limited approaches that fail at depth 10,000 (use an explicit stack).

## Edge cases

- Empty input -> [].
- Sibling order preserved.

## Examples

- [{id:1,children:[{id:2},{id:3,children:[{id:4}]}]}] -> [{id:1,depth:0,parentId:null,path:[1]},{id:2,depth:1,parentId:1,path:[1,2]},{id:3,depth:1,parentId:1,path:[1,3]},{id:4,depth:2,parentId:3,path:[1,3,4]}]
- [] -> []
- [{id:'a',name:'x',children:null}] -> [{id:'a',name:'x',depth:0,parentId:null,path:['a']}]
