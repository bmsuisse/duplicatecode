export type FlatNode<T> = Omit<T, "children"> & {
  depth: number;
  parentId: string | number | null;
  path: Array<string | number>;
};

export function flattenTree<T extends { id: string | number; children?: T[] | null }>(
  roots: readonly T[],
): FlatNode<T>[] {
  const result: FlatNode<T>[] = [];

  interface StackItem {
    node: T;
    depth: number;
    parentId: string | number | null;
    path: Array<string | number>;
  }

  const stack: StackItem[] = [];

  // Push all roots in reverse order so they're processed in correct order
  for (let i = roots.length - 1; i >= 0; i--) {
    stack.push({
      node: roots[i],
      depth: 0,
      parentId: null,
      path: [],
    });
  }

  while (stack.length > 0) {
    const item = stack.pop();
    if (!item) break;

    const { node, depth, parentId, path } = item;

    const nodePath = [...path, node.id];

    // Create the flat node
    const flatNode = {
      ...(node as unknown as Record<string, unknown>),
      depth,
      parentId,
      path: nodePath,
    };

    // Remove children from flatNode
    delete flatNode.children;

    result.push(flatNode as FlatNode<T>);

    // Push children in reverse order
    const children = node.children;
    if (children && Array.isArray(children)) {
      for (let i = children.length - 1; i >= 0; i--) {
        stack.push({
          node: children[i],
          depth: depth + 1,
          parentId: node.id,
          path: nodePath,
        });
      }
    }
  }

  return result;
}
