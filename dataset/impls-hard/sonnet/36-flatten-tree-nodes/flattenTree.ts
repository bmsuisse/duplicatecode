/** Return all nodes in pre-order (parents before their children, siblings left to right). */
export function flattenTree<N>(
  roots: N | readonly N[],
  getChildren: (node: N) => readonly N[] | undefined | null,
): N[] {
  const result: N[] = [];
  const stack: N[] = (Array.isArray(roots) ? [...(roots as readonly N[])] : [roots as N]).reverse();
  while (stack.length > 0) {
    const node = stack.pop() as N;
    result.push(node);
    const children = getChildren(node);
    if (children) {
      for (let i = children.length - 1; i >= 0; i--) stack.push(children[i] as N);
    }
  }
  return result;
}

/** Flatten nodes that keep their children in a `children` property. */
export function flattenChildren<N extends { children?: readonly N[] }>(
  roots: N | readonly N[],
): N[] {
  return flattenTree(roots, (node) => node.children);
}
