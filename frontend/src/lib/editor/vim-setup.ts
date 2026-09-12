import type { EditorView } from "codemirror";
import { Compartment } from "@codemirror/state";

import { Vim, vim } from "@replit/codemirror-vim";
import type * as Y from "yjs";

export const vimCompartment = new Compartment();

export function createVim(undoManager: Y.UndoManager) {
  // Creates commands for Yjs compatibale undos
  Vim.defineEx("yundo", "yu", () => {
    undoManager.undo();
  });
  Vim.defineEx("yredo", "yr", () => {
    undoManager.redo();
  });

  // Overwrite standard undo with the Yjs ones
  Vim.map("u", ":yundo<CR>", "normal");
  Vim.map("<C-r>", ":yredo<CR>", "normal");

  // Remove vim bindings that conflict with the formatting shortcuts
  Vim.unmap("<C-u>", undefined as any);
  Vim.unmap("<C-i>", undefined as any);
  Vim.unmap("<C-b>", undefined as any);

  return vimCompartment.of(vim());
}

export function setVimEnabled(view: EditorView, enabled: boolean) {
  view.dispatch({
    effects: vimCompartment.reconfigure(enabled ? vim() : []),
  });
}
