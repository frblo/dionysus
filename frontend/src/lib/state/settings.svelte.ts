export const userSettings = $state({
  vimEnabled: true,
  highlighTrailingSpacesEnabled: true,
});

export enum SidebarMenus {
  Outline,
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
