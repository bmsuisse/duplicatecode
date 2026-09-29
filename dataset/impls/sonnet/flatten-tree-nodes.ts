export type FlatNode<T> = Omit<T, "children"> & {
  depth: number;
  parentId: string | number | null;
  path: Array<string | number>;
};

export function flattenTree<T extends { id: string | number; children?: T[] | null }>(
  roots: readonly T[],
): FlatNode<T>[] {
  type Frame = {
    node: T;
    depth: number;
    parentId: string | number | null;
    path: Array<string | number>;
  };
  const output: FlatNode<T>[] = [];
  const stack: Frame[] = [];
  for (let i = roots.length - 1; i >= 0; i--) {
    stack.push({ node: roots[i], depth: 0, parentId: null, path: [] });
  }
  while (stack.length > 0) {
    const frame = stack.pop() as Frame;
    const { children, ...rest } = frame.node;
    const path = [...frame.path, frame.node.id];
    output.push({
      ...rest,
      depth: frame.depth,
      parentId: frame.parentId,
      path,
    } as unknown as FlatNode<T>);
    if (children) {
      for (let i = children.length - 1; i >= 0; i--) {
        stack.push({ node: children[i], depth: frame.depth + 1, parentId: frame.node.id, path });
      }
    }
  }
  return output;
}
