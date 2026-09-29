export interface TreeNode<T> {
  value: T;
  children?: TreeNode<T>[];
}

export function flattenTree<T>(node: TreeNode<T>): T[] {
  const result: T[] = [node.value];

  if (node.children) {
    for (const child of node.children) {
      result.push(...flattenTree(child));
    }
  }

  return result;
}
