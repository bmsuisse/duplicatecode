export interface TreeNode {
  children?: readonly TreeNode[] | null;
}

export type FlatNode<T extends TreeNode> = Omit<T, "children"> & {
  depth: number;
  parent: FlatNode<T> | null;
};

/**
 * Flatten a tree depth-first (parents before children). Each entry is a copy of the
 * node without `children`, plus its `depth` and a reference to its flattened `parent`.
 */
export function flattenTree<T extends TreeNode>(roots: T | readonly T[]): FlatNode<T>[] {
  const result: FlatNode<T>[] = [];
  const list: readonly T[] = Array.isArray(roots) ? roots : [roots as T];
  const stack: { node: T; depth: number; parent: FlatNode<T> | null }[] = [];
  for (let i = list.length - 1; i >= 0; i--) stack.push({ node: list[i], depth: 0, parent: null });

  while (stack.length > 0) {
    const item = stack.pop();
    if (!item) break;
    const { children, ...rest } = item.node;
    const flat = { ...rest, depth: item.depth, parent: item.parent } as unknown as FlatNode<T>;
    result.push(flat);
    const kids = (children ?? []) as readonly T[];
    for (let i = kids.length - 1; i >= 0; i--) {
      stack.push({ node: kids[i], depth: item.depth + 1, parent: flat });
    }
  }
  return result;
}
