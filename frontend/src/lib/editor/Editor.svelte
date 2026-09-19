<script lang="ts">
  import { onMount, tick } from "svelte";
  import { syntaxHighlighting } from "@codemirror/language";
  import { basicDark } from "@fsegurai/codemirror-theme-basic-dark";
  import { basicSetup } from "codemirror";
  import { EditorView, keymap } from "@codemirror/view";
  import { EditorState, EditorSelection } from "@codemirror/state";
  import { fountain, fountainHighlightStyle } from "$lib/fountain-highlight";

  import * as Y from "yjs";
  import { WebsocketProvider } from "y-websocket";
  import { yCollab } from "y-codemirror.next";

  import { createVim, setVimEnabled } from "$lib/editor/vim-setup";
  import { userSettings } from "$lib/state/settings.svelte";
  import { preview } from "$lib/state/preview.svelte";
  import { editor } from "$lib/state/editor.svelte";
  import { sceneScanner } from "$lib/state/scenes.svelte";
  import {
    createTrailingSpaces,
    setTrailingSpacesEnabled,
  } from "$lib/editor/trailing-spaces";
  import { debounce } from "$lib/utils/debounce";

  // Decide on what protocol to use based on if its https or http
  const proto = location.protocol === "https:" ? "wss:" : "ws:";
  const wsUrl = `${proto}//${location.host}/rooms/ws`;

  let {
    room = "",
    user = {
      name: "Anonymous" + Math.floor(Math.random() * 100),
      color: "#30bced",
    },
  } = $props();

  let editorEl: HTMLDivElement;

  let view: EditorView | null = null;
  let provider: WebsocketProvider | null = null;
  let undoManager: Y.UndoManager | null = null;

  onMount(() => {
    const ydoc = new Y.Doc();
    provider = new WebsocketProvider(wsUrl, room, ydoc);
    const ytext = ydoc.getText("codemirror");

    undoManager = new Y.UndoManager(ytext);

    provider.awareness.setLocalStateField("user", user);

    const vimExt = createVim(undoManager);
    const trailingSpaces = createTrailingSpaces();
    const debouncedPreview = debounce(async (text: string, line: number) => {
      preview.generatePreview(text);
      await tick();
      preview.scrollToLine(line);
    }, 50);

    view = new EditorView({
      parent: editorEl,
      state: EditorState.create({
        doc: ytext.toString(),
        extensions: [
          fountain(),
          syntaxHighlighting(fountainHighlightStyle),
          basicDark,
          trailingSpaces,
          yCollab(ytext, provider.awareness, { undoManager }),
          vimExt,
          keymap.of([
            {
              key: "Mod-z",
              run: () => {
                undoManager?.undo();
                return true;
              },
            },
            {
              key: "Mod-Shift-z",
              run: () => {
                undoManager?.redo();
                return true;
              },
            },
            {
              key: "Mod-u",
              run: () => {
                surroundSelection("_", "_");
                return true;
              },
            },
            {
              key: "Mod-i",
              run: () => {
                surroundSelection("*", "*");
                return true;
              },
            },
            {
              key: "Mod-b",
              run: () => {
                surroundSelection("**", "**");
                return true;
              },
            },
            {
              key: "Mod-/",
              run: () => {
                toggleBoneyard();
                return true;
              },
            },
          ]),
          EditorView.lineWrapping,
          EditorView.contentAttributes.of({ spellcheck: "true" }),
          sceneScanner,
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              const head = update.state.selection.main.head;
              debouncedPreview(
                update.state.doc.toString(),
                update.state.doc.lineAt(head).number,
              );
            }
          }),
          basicSetup,
        ],
      }),
    });

    provider.on("status", (e) => {
      console.log(`[yws] ${e.status} ${wsUrl}/${room}`);
    });

    return () => {
      view?.destroy();
      provider?.destroy();
      ydoc.destroy();
    };
  });

  $effect(() => {
    if (!view) return;
    setVimEnabled(view, userSettings.vimEnabled);
    setTrailingSpacesEnabled(view, userSettings.highlighTrailingSpacesEnabled);
  });

  $effect(() => {
    editor.scrollTick;
    jumpToLine(editor.targetLine);
  });

  export function getContent() {
    return view ? view.state.doc.toString() : "";
  }

  export function getCursorLine() {
    if (!view) return 0;
    const head = view.state.selection.main.head;
    return view.state.doc.lineAt(head).number;
  }

  export function jumpToLine(line: number) {
    if (!view) return;

    const pos = view.state.doc.line(line).from;
    view.dispatch({
      selection: { anchor: pos, head: pos },
      effects: EditorView.scrollIntoView(pos, { y: "start" }),
    });
    view.focus();
  }

  export function undo() {
    undoManager?.undo();
  }

  export function redo() {
    undoManager?.redo();
  }

  export function toggleBoneyard() {
    if (!view) return;

    const { from, to } = view.state.selection.main;
    if (from === to) {
      view.dispatch({
        changes: { from, insert: "/*  */" },
        selection: { anchor: from + 3 },
      });
    } else {
      const selected = view.state.sliceDoc(from, to);
      const wrapped = selected.startsWith("/* ") && selected.endsWith(" */");
      const markerLength = 3;

      view.dispatch({
        changes: wrapped
          ? [
              { from, to: from + markerLength, insert: "" },
              { from: to - markerLength, to, insert: "" },
            ]
          : [
              { from, insert: "/* " },
              { from: to, insert: " */" },
            ],
        selection: wrapped
          ? { anchor: from, head: to - markerLength * 2 }
          : { anchor: from + markerLength, head: to + markerLength },
      });
    }

    view.focus();
  }

  export function surroundSelection(prefix: string, suffix: string) {
    if (!view) return;
    const state = view.state;
    const prefixLength = prefix.length;
    const changes: { from: number; to: number; insert: string }[] = [];
    const selections: { anchor: number; head: number }[] = [];
    let offset = 0;

    for (const range of state.selection.ranges) {
      const from = range.from;
      const to = range.to;
      const length = to - from;

      if (length === 0) {
        const line = state.doc.lineAt(from);
        const lineFrom = line.from;
        const lineText = line.text;
        const pos = from - lineFrom;

        let open = -1;
        for (let i = pos - prefixLength; i >= 0; i--) {
          if (lineText.slice(i, i + prefixLength) === prefix) {
            open = i;
            break;
          }
        }

        let close = -1;
        if (
          open !== -1 &&
          !lineText.slice(open + prefixLength, pos).includes(suffix)
        ) {
          const remaining = lineText.slice(pos);
          const idx = remaining.indexOf(suffix);
          if (idx !== -1 && !lineText.slice(pos, pos + idx).includes(prefix)) {
            close = pos + idx;
          }
        }

        if (close !== -1) {
          changes.push({
            from: lineFrom + open,
            to: lineFrom + open + prefixLength,
            insert: "",
          });
          changes.push({
            from: lineFrom + close,
            to: lineFrom + close + prefixLength,
            insert: "",
          });
          selections.push({
            anchor: from - prefixLength + offset,
            head: from - prefixLength + offset,
          });
          offset -= 2 * prefixLength;
        } else {
          changes.push({ from, to, insert: prefix + suffix });
          selections.push({
            anchor: from + offset + prefixLength,
            head: from + offset + prefixLength,
          });
          offset += 2 * prefixLength;
        }
      } else {
        const wrappedOutside =
          from - prefixLength >= 0 &&
          to + prefixLength <= state.doc.length &&
          state.sliceDoc(from - prefixLength, from) === prefix &&
          state.sliceDoc(to, to + prefixLength) === suffix;

        if (wrappedOutside) {
          changes.push({ from: from - prefixLength, to: from, insert: "" });
          changes.push({ from: to, to: to + prefixLength, insert: "" });
          selections.push({
            anchor: from - prefixLength + offset,
            head: from - prefixLength + offset + length,
          });
          offset -= 2 * prefixLength;
        } else {
          const wrappedInside =
            prefixLength < length &&
            state.sliceDoc(from, from + prefixLength) === prefix &&
            state.sliceDoc(to - prefixLength, to) === suffix;

          if (wrappedInside) {
            changes.push({ from, to: from + prefixLength, insert: "" });
            changes.push({ from: to - prefixLength, to, insert: "" });
            selections.push({
              anchor: from + offset,
              head: from + offset + length - 2 * prefixLength,
            });
            offset -= 2 * prefixLength;
          } else {
            const selected = state.sliceDoc(from, to);
            changes.push({ from, to, insert: prefix + selected + suffix });
            selections.push({
              anchor: from + offset + prefixLength,
              head: from + offset + prefixLength + selected.length,
            });
            offset += 2 * prefixLength;
          }
        }
      }
    }

    view.dispatch({
      changes,
      selection: EditorSelection.create(
        selections.map((s) => EditorSelection.range(s.anchor, s.head)),
      ),
    });
    view.focus();
  }
</script>

<div bind:this={editorEl} class="editor"></div>
