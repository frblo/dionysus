export const userSettings = $state({
  vimEnabled: true,
  highlighTrailingSpacesEnabled: true,
});

export enum SidebarMenus {
  Outline,
  None,
}

export const editorViewSettings = $state({
  sidebarMenuOpen: SidebarMenus.None,
  exportMenuOpen: false,
});
