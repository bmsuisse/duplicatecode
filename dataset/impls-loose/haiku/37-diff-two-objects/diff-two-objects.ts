export interface Diff {
  added: Record<string, any>;
  removed: Record<string, any>;
  changed: Record<string, { old: any; new: any }>;
}

export function diffObjects(old: Record<string, any>, neu: Record<string, any>): Diff {
  const added: Record<string, any> = {};
  const removed: Record<string, any> = {};
  const changed: Record<string, any> = {};

  for (const key in neu) {
    if (!(key in old)) {
      added[key] = neu[key];
    } else if (old[key] !== neu[key]) {
      changed[key] = { old: old[key], new: neu[key] };
    }
  }

  for (const key in old) {
    if (!(key in neu)) {
      removed[key] = old[key];
    }
  }

  return { added, removed, changed };
}
