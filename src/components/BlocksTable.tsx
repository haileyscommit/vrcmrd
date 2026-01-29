import { users } from "../data/users";

export function BlockingModsTable() {
    return (  
    <div class="flex-1 min-h-0 min-w-0 w-full overflow-hidden bg-gray-100 dark:bg-gray-900">
      <div class="h-full w-full min-h-0 min-w-0 overflow-y-scroll overflow-x-hidden">
        <table class="w-full text-sm text-left text-gray-700 dark:text-gray-200">
          <thead class="bg-gray-50 dark:bg-gray-800">
            <tr class="text-xs text-gray-500 uppercase">
                <th class="px-2 py-2">User</th>
                <th class="px-2 py-2">Blocked moderator</th>
            </tr>
          </thead>
          <tbody>
              {/* TODO: smart sort: user-specific advisories, avatar advisories, group membership advisories. Within each category, users should be sorted by most recent leave or join first */}
            {users.map((u, idx) => (
              <tr
                key={u.id}
                class={(idx % 2 === 0 ? 'bg-white dark:bg-transparent' : 'bg-gray-50 dark:bg-gray-800/30') + ' h-[32px] hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer active:bg-gray-50 dark:active:bg-black/50'}
              //   onContextMenu={async (e) => {
              //     e.preventDefault();
              //     const cm = await menu.Menu.new({id: `ucm-${u.id}`, items: [
              //       await menu.MenuItem.new({
              //         text: 'Log warning'
              //       }),
              //       await menu.MenuItem.new({
              //         text: 'Add shared note'
              //       }),
              //       await menu.MenuItem.new({
              //         text: 'Create moderation ticket'
              //       }),
              //       await menu.MenuItem.new({
              //         text: 'View profile in browser'
              //       }),
              //       await menu.MenuItem.new({
              //         text: 'Copy user ID'
              //       }),
              //     ]});
              //     cm.popup();
              //   }}
                >
                <td class="px-2 py-1 align-middle">
                    <div class="flex items-center gap-3">
                    <div class="flex flex-col">
                      <div class="font-medium text-sm">{u.username}</div>
                    </div>
                    </div>
                </td>
                <td class="px-2 py-1 align-middle">
                    <div class="text-xs text-gray-500">{u.avatarName}</div>
                </td>
                </tr>
            ))}
            </tbody>
        </table>
      </div>
    </div>
  );
}