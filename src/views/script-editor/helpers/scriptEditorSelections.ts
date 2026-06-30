export const selectCurrentItem = <T extends { id: string }>(items: T[], selectedId: string | null | undefined): T | null => {
  if (!selectedId) {
    return items[0] ?? null;
  }
  return items.find((item) => item.id === selectedId) ?? items[0] ?? null;
};
