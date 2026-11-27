import { syntaxHighlighting } from "@codemirror/language";
import { basicDark } from "@fsegurai/codemirror-theme-basic-dark";
import { basicSetup } from "codemirror";
import { EditorView } from "@codemirror/view";
import { EditorState, type Extension } from "@codemirror/state";

import { fountain, fountainHighlightStyle } from "$lib/fountain-highlight";
import {
  createTrailingSpaces,
  setTrailingSpacesEnabled,
} from "$lib/editor/trailing-spaces";
import { setVimEnabled } from "$lib/editor/vim-setup";
import { userSettings } from "$lib/state/settings.svelte";

type CreateEditorViewOptions = {
  parent: HTMLElement;
  doc: string;
  readOnly?: boolean;
  extensions?: Extension[];
};

export function createEditorView({
  parent,
  doc,
  readOnly = false,
  extensions = [],
}: CreateEditorViewOptions): EditorView {
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc,
      extensions: [
        fountain(),
        syntaxHighlighting(fountainHighlightStyle),
        basicDark,
        createTrailingSpaces(),
        EditorView.lineWrapping,
        EditorView.contentAttributes.of({ spellcheck: "true" }),
        EditorState.readOnly.of(readOnly),
        ...extensions,
        basicSetup,
      ],
    }),
  });

  applyEditorSettings(view);
  return view;
}

export function applyEditorSettings(view: EditorView) {
  setVimEnabled(view, userSettings.current.vimEnabled);
  setTrailingSpacesEnabled(
    view,
    userSettings.current.highlighTrailingSpacesEnabled,
  );
}

export function getEditorContent(view: EditorView | null): string {
  return view ? view.state.doc.toString() : "";
}
