import { emit } from "@tauri-apps/api/event";

export default function Menubar() {
  return <div id="menubar" class="h-7 px-2 gap-2 flex flex-row w-full bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
    <MenubarButton label="Refresh" onClick={() => {
      emit('vrcmrd:cache_refresh', {});
      document.dispatchEvent(new CustomEvent('vrcmrd:soft-refresh'));
      /* refresh caches logic */
    }} />
    <MenubarButton label="Manage access" onClick={() => { /* manage users logic */ }} />
    <MenubarButton label="Manage watchlists" onClick={() => { /* manage watched groups/users logic */ }} />
    <MenubarButton label="Settings" onClick={() => { /* open settings window */ }} />
  </div>;
}
function MenubarButton(props: { label: string; onClick: () => void }) {
  return <button
    type="button"
    className="h-full px-2 text-xs font-medium bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 active:bg-gray-300 dark:active:bg-gray-600 text-gray-700 dark:text-gray-200 flex items-center"
    aria-label={props.label}
    onClick={props.onClick}
  >
    {props.label}
  </button>
}