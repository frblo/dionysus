import { ViewPlugin, ViewUpdate } from "@codemirror/view";
import { ensureSyntaxTree, syntaxTree } from "@codemirror/language";

export interface Scene {
  name: string;
  pos: number;
}

export const scenes = $state({
  list: <Scene[]>([])
});

export const sceneScanner = ViewPlugin.fromClass(
  class {
    update(update: ViewUpdate) {
      if (update.docChanged) {
        this.scanScenes(update);
      }
    }

    scanScenes(update: ViewUpdate) {
      const foundScenes: { name: string; pos: number }[] = [];
      const state = update.state;

      const tree = ensureSyntaxTree(state, state.doc.length, 50) || syntaxTree(state);

      tree.iterate({
        enter(node) {
          if (node.name === "scene_heading") {
            foundScenes.push({
              name: state.sliceDoc(node.from, node.to),
              pos: node.from,
            });
          }
        },
      });

      scenes.list = foundScenes;
    }
  },
);

