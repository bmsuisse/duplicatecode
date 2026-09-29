export interface Diff {
  added: Record<string, any>;
  removed: Record<string, any>;
  changed: Record<string, { old: any; new: any }>;
}

export function diffObjects(obj1: Record<string, any>, obj2: Record<string, any>): Diff {
  const diff: Diff = { added: {}, removed: {}, changed: {} };
  const allKeys = new Set([...Object.keys(obj1), ...Object.keys(obj2)]);

  for (const key of allKeys) {
    const in1 = key in obj1;
    const in2 = key in obj2;

    if (!in1 && in2) {
      diff.added[key] = obj2[key];
    } else if (in1 && !in2) {
      diff.removed[key] = obj1[key];
    } else if (in1 && in2 && obj1[key] !== obj2[key]) {
      diff.changed[key] = { old: obj1[key], new: obj2[key] };
    }
  }

  return diff;
}
