import { LocalStorage } from "$lib/utils/storage.svelte";

type UserSettings = {
  vimEnabled: boolean;
  highlighTrailingSpacesEnabled: boolean;
};

export const userSettings = new LocalStorage<UserSettings>(
  "dionysus:userSettings",
  {
    vimEnabled: true,
    highlighTrailingSpacesEnabled: true,
  },
);

export enum SidebarMenus {
  Outline,
  Settings,
  None,
}

export enum PanelFocus {
  Both,
  EditorOnly,
  PreviewOnly,
}

export const editorViewSettings = $state({
  sidebarMenuOpen: SidebarMenus.None,
  exportMenuOpen: false,
  panelFocus: PanelFocus.Both,
});
