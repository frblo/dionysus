export const userSettings = $state({
  vimEnabled: true,
});

export enum SidebarMenus {
  Outline,
}

export const editorViewSettings = $state({
  outlineOpen: false,
});
