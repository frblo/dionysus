export const userSettings = $state({
  vimEnabled: true,
});

export enum SidebarMenus {
  Outline,
  None,
}

export const editorViewSettings = $state({
  sidebarMenuOpen: SidebarMenus.None,
  exportMenuOpen: false,
});
