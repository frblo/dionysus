import { Compartment } from "@codemirror/state";
import { highlightTrailingWhitespace, type EditorView } from "@codemirror/view";

export const trailingSpaces = new Compartment;

export function createTrailingSpaces() {
  return trailingSpaces.of(highlightTrailingWhitespace());
}

export function setTrailingSpacesEnabled(view: EditorView, enabled: boolean) {
  view.dispatch({
    effects: trailingSpaces.reconfigure(enabled ? highlightTrailingWhitespace() : []),
  });
}
