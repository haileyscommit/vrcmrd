import { Menu, MenuButton, MenuItem, MenuItems } from "@headlessui/react";
import DownArrowIcon from "mdi-preact/MenuDownIcon";

export default function Dropdown({ items, class: className, label }: { items: { active: boolean, set: () => void, label: preact.VNode }[], class?: string, label?: preact.VNode }) {
  const menuItemClasses = (active: boolean) => `w-full text-left p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg ${active ? 'bg-gray-100 dark:bg-gray-700' : ''}`;;
  return <div class={`relative ${className ?? ""}`}>
    <Menu>
      <MenuButton class="w-full p-2 border border-gray-300 dark:border-gray-700 rounded text-left flex justify-between items-center">
        {label ?? items.find(item => item.active)?.label}
        <DownArrowIcon />
      </MenuButton>
      <MenuItems anchor={{to: "bottom", padding: 24}} class="absolute bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 text-black dark:text-white rounded mt-1 w-full max-h-60 overflow-y-auto z-60">
        {items.map((item, i) => <MenuItem key={i}>
          <button class={menuItemClasses(item.active)} onClick={() => !item.active && item.set()}>{item.label}</button>
        </MenuItem>)}
      </MenuItems>
    </Menu>
  </div>;
}