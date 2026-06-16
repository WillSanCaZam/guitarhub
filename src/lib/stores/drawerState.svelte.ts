let open = $state(false);

export function drawerState() {
  return {
    get open() {
      return open;
    },
    toggle() {
      open = !open;
    },
    close() {
      open = false;
    },
    openDrawer() {
      open = true;
    }
  };
}