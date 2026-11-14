// 功能：网页预览使用同一快捷键动作表，桌面版交给原生菜单以避免重复执行；时间：2026-11-14；作者：lq。
export function keyboardAction(event: Pick<KeyboardEvent, 'key' | 'ctrlKey' | 'metaKey' | 'shiftKey' | 'altKey' | 'isComposing'>): string | undefined {
  if (event.isComposing || event.altKey) return;
  const key = event.key.toLowerCase();
  if (event.ctrlKey && key === 'tab') return event.shiftKey ? 'previous-tab' : 'next-tab';
  if (!(event.metaKey || event.ctrlKey)) return;
  if (key === 't') return event.shiftKey ? 'restore-tab' : 'new-tab';
  if (key === 'a' && event.shiftKey) return 'tab-picker';
  if (key === 'o' && event.shiftKey) return 'bookmarks';
  if (key === 'j' && event.shiftKey) return 'downloads';
  return ({ l: 'focus-address', w: 'close-tab', d: 'save-bookmark', y: 'history', ',': 'settings', r: 'reload', '+': 'zoom-in', '=': 'zoom-in', '-': 'zoom-out', '0': 'zoom-reset', q: 'quit' } as Record<string, string>)[key];
}
