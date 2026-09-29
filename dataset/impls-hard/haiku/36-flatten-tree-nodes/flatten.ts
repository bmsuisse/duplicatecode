export interface TreeNode<T> {
  data: T;
  children?: TreeNode<T>[];
}

export function flattenTree<T>(root: TreeNode<T>): T[] {
  const result: T[] = [];

  function traverse(node: TreeNode<T>): void {
    result.push(node.data);
    if (node.children) {
      for (const child of node.children) {
        traverse(child);
      }
    }
  }

  traverse(root);
  return result;
}
