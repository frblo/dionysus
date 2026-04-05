<script lang="ts">
  import { onMount } from "svelte";
  import { syntaxHighlighting } from "@codemirror/language";
  import { basicDark } from "@fsegurai/codemirror-theme-basic-dark";
  import { basicSetup } from "codemirror";
  import { EditorView, keymap } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";
  import { fountain, fountainHighlightStyle } from "$lib/fountain-highlight";

  import * as Y from "yjs";
  import { WebsocketProvider } from "y-websocket";
  import { yCollab } from "y-codemirror.next";

  import { createVim, setVimEnabled } from "$lib/editor/vim-setup";
  import { userSettings } from "$lib/state/settings.svelte";
  import { preview } from "$lib/state/preview.svelte";
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
    room = "demo-room-1",
    user = {
      name: "Anonymous" + Math.floor(Math.random() * 100),
      color: "#30bced",
    },
  } = $props();

  let editorEl: HTMLDivElement;

  let view: EditorView | null = null;
  let provider: WebsocketProvider | null = null;

  onMount(() => {
    const ydoc = new Y.Doc();
    provider = new WebsocketProvider(wsUrl, room, ydoc);
    const ytext = ydoc.getText("codemirror");

    const undoManager = new Y.UndoManager(ytext);

    provider.awareness.setLocalStateField("user", user);

    const vimExt = createVim(undoManager, updatePreview);
    const trailingSpaces = createTrailingSpaces();
    const debouncedPreview = debounce((text: string, line: number) => {
      preview.generatePreview(text);
      preview.scrollToLine(line);
    }, 300);

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
                undoManager.undo();
                return true;
              },
            },
            {
              key: "Mod-Shift-z",
              run: () => {
                undoManager.redo();
                return true;
              },
            },
            {
              key: "Mod-s",
              run: () => {
                updatePreview();
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

  export function getContent() {
    return view ? view.state.doc.toString() : "";
  }

  export function getCursorLine() {
    if (!view) return 0;
    const head = view.state.selection.main.head;
    return view.state.doc.lineAt(head).number;
  }

  export function scrollIntoView(pos: number) {
    if (!view) {
      return;
    }
    view.dispatch({
      effects: EditorView.scrollIntoView(pos, { y: "start", x: "start" }),
    });
    view.focus();
  }

  export function updatePreview() {
    const script = getContent();
    preview.generatePreview(script);
  }
</script>

<div bind:this={editorEl} class="editor"></div>
