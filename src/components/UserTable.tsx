import { User, users } from '../data/users';
import AlertIcon from "mdi-preact/AlertIcon";
import CardAccountDetailsIcon from "mdi-preact/CardAccountDetailsIcon";
import AndroidIcon from "mdi-preact/AndroidIcon";
import AppleIcon from "mdi-preact/AppleIcon";
import MonitorIcon from "mdi-preact/MonitorIcon";
import { useEffect, useState } from 'preact/hooks';
import { listen } from '@tauri-apps/api/event';
import { menu } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';

export default function UserTable() {
  const [userList, setUserList] = useState<User[]>(users); // placeholder for future dynamic data
  useEffect(() => {
    const initialUsers = async () => {
      invoke<User[]>('get_users').then(fetchedUsers => {
        setUserList(fetchedUsers);
      });
    };
    initialUsers();
    const joinUnlisten = listen('vrcmrd:join', (event) => {
      // TODO: make it match the User struct on the Rust side
      const detail = event.payload as User;
      setUserList(prev => [...prev, detail]);
    });
    const leaveUnlisten = listen('vrcmrd:leave', (event) => {
      const detail = event.payload as User;
      setUserList(prev => prev.map(u => {
        if (u.id === detail.id) {
          return { ...u, leaveTime: Date.now().toString() };
        }
        return u;
      }));
    });

    return () => {
      joinUnlisten.then(unlisten => unlisten());
      leaveUnlisten.then(unlisten => unlisten());
    };
  }, []);
  return (  
    <div class="flex-grow w-screen overflow-x-auto overflow-y-scroll bg-gray-100 dark:bg-gray-900">
      <table class="min-w-full text-sm text-left text-gray-700 dark:text-gray-200">
        <thead class="bg-gray-50 dark:bg-gray-800">
          <tr class="text-xs text-gray-500 uppercase">
              <th class="px-2 py-2">User</th>
              <th class="px-2 py-2">Avatar</th>
              <th class="px-2 py-2">Perf</th>
              <th class="px-2 py-2">Acc Age</th>
              <th class="px-2 py-2">Join Time</th>
              <th class="px-2 py-2">Leave Time</th>
              <th class="px-2 py-2 text-right">Flags</th>
          </tr>
        </thead>
        <tbody>
            {/* TODO: smart sort: user-specific advisories, avatar advisories, group membership advisories. Within each category, users should be sorted by most recent leave or join first */}
          {userList.map((u, idx) => (
            <tr
              key={u.id}
              class={(idx % 2 === 0 ? 'bg-white dark:bg-transparent' : 'bg-gray-50 dark:bg-gray-800/30') + '  hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer active:bg-gray-50 dark:active:bg-black/50'}
              onContextMenu={async (e) => {
                e.preventDefault();
                const cm = await menu.Menu.new({id: `ucm-${u.id}`, items: [
                  await menu.MenuItem.new({
                    text: 'Log warning'
                  }),
                  await menu.MenuItem.new({
                    text: 'Add shared note'
                  }),
                  await menu.MenuItem.new({
                    text: 'Create moderation ticket'
                  }),
                  await menu.MenuItem.new({
                    text: 'View profile in browser'
                  }),
                  await menu.MenuItem.new({
                    text: 'Copy user ID'
                  }),
                ]});
                cm.popup();
              }}
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
              <td class="px-2 py-1 align-middle font-semibold">{/* TODO: performance rank icons */ u.perfRank}</td>
              <td class="px-2 py-1 align-middle text-xs text-gray-500">{u.accountAge}</td>
              <td class="px-2 py-1 align-middle text-xs">{u.joinTime}</td>
              <td class="px-2 py-1 align-middle text-xs">{u.leaveTime}</td>
              <td class="px-2 py-1 align-middle">
                  <div class="flex items-center justify-end gap-2">
                  {u.advisories && (
                      <div class="tooltip right" aria-hidden>
                      <AlertIcon />
                      <div class="tooltip-tip">Advisories added by user</div>
                      </div>
                  )}

                  {u.ageVerified && (
                      <div class="tooltip right" aria-hidden>
                      <CardAccountDetailsIcon />
                      <div class="tooltip-tip">Age verified</div>
                      </div>
                  )}

                  <div class="tooltip right" aria-hidden>
                      {u.platform === 'pc' && <MonitorIcon />}
                      {u.platform === 'android' && <AndroidIcon />}
                      {u.platform === 'ios' && <AppleIcon />}
                      <div class="tooltip-tip">Platform: {u.platform.toUpperCase()}</div>
                  </div>
                  </div>
              </td>
              </tr>
          ))}
          </tbody>
      </table>
      </div>
  );
}
