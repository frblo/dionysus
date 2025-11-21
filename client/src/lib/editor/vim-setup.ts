import { Vim, vim } from "@replit/codemirror-vim";
import type * as Y from "yjs";

export function createVim(undoManager: Y.UndoManager) {
  Vim.defineEx("yundo", "yu", () => {
    undoManager.undo();
  });

  Vim.defineEx("yredo", "yr", () => {
    undoManager.redo();
  });

  Vim.map("u", ":yundo<CR>", "normal");
  Vim.map("<C-r>", ":yredo<CR>", "normal");

  return vim();
}
