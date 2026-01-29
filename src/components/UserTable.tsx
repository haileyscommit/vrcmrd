import { User } from '../data/users';
import AlertIcon from "mdi-preact/AlertIcon";
import CardAccountDetailsIcon from "mdi-preact/CardAccountDetailsIcon";
import AndroidIcon from "mdi-preact/AndroidIcon";
import AppleIcon from "mdi-preact/AppleIcon";
import MonitorIcon from "mdi-preact/MonitorIcon";
import HelpOutlineIcon from "mdi-preact/HelpCircleOutlineIcon";
import { useEffect, useState } from 'preact/hooks';
import { listen } from '@tauri-apps/api/event';
import { menu } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';

export default function UserTable() {
  const [userList, setUserListImpl] = useState<User[]>([]);
  const setUserList = (f: ((prev: User[]) => User[]) | User[]) => {
    // Sort the users here, so we don't have to re-sort on every render.
    setUserListImpl(prev => (typeof f === 'function' ? f(prev) : f)
      .sort((a, b) => {
        // Sort by most recent join/leave time
        const aTime = a.leaveTime ?? a.joinTime;
        const bTime = b.leaveTime ?? b.joinTime;
        return bTime - aTime;
      })
      .sort((a, b) => {
        // Users currently in-world first (leaveTime === null)
        if (a.leaveTime === null && b.leaveTime !== null) return -1;
        if (a.leaveTime !== null && b.leaveTime === null) return 1;
        return 0;
      })
    );
  }
  useEffect(() => {
    var currentInstanceId: string | null = null;
    // This effect pre-populates the page and handles events from the backend that update the user list.
    const initialUsers = async () => {
      invoke<User[]>('get_users').then(fetchedUsers => {
        setUserList(fetchedUsers);
      });
    };
    initialUsers();
    const joinUnlisten = listen('vrcmrd:join', (event) => {
      const detail = event.payload as User;
      const index = userList.findIndex(u => u.id === detail.id);
      if (index !== -1) {
        // already in list
        userList[index] = detail;
        setUserList([...userList]);
        return;
      }
      setUserList(prev => [...prev, detail]);
    });
    const instanceUnlisten = listen('vrcmrd:instance', (event) => {
      setUserList([]);
      currentInstanceId = event.payload as string;
      setTimeout(() => {
        if (currentInstanceId === event.payload as string) {
          softRefreshList();
        }
      }, 2000);
    });
    const leaveUnlisten = listen('vrcmrd:leave', (event) => {
      const detail = event.payload as User;
      const index = userList.findIndex(u => u.id === detail.id);
      if (index === -1) {
        // not in list; add them
        setUserList(prev => [...prev, detail]);
        return;
      }
      setUserList(prev => prev.map(u => {
        if (u.id === detail.id) {
          return { ...u, leaveTime: detail.leaveTime ?? (Date.now() / 1000) }; // assuming leaveTime is a unix timestamp in seconds
        }
        return u;
      }));
    });
    const updateUnlisten = listen('vrcmrd:update-user', (event) => {
      const detail = event.payload as User;
      if (!userList.find(u => u.id === detail.id)) {
        //setUserList(prev => [...prev, detail]);
        return;
      }
      setUserList(prev => prev.map(u => u.id === detail.id ? detail : u));
    });
    const softRefreshList = () => {
      invoke<User[]>('get_users').then(fetchedUsers => {
        setUserList(fetchedUsers);
      });
    };
    document.addEventListener('vrcmrd:soft-refresh', softRefreshList);
    const refreshListInterval = setInterval(softRefreshList, 15 * 1000); // every 15 seconds

    return () => {
      joinUnlisten.then(unlisten => unlisten());
      leaveUnlisten.then(unlisten => unlisten());
      updateUnlisten.then(unlisten => unlisten());
      instanceUnlisten.then(unlisten => unlisten());
      document.removeEventListener('vrcmrd:soft-refresh', softRefreshList);
      clearInterval(refreshListInterval);
    };
  }, []);
  return (
    // Shrinkable container prevents layout collapse on tab switches.
    <div class="flex-1 min-h-0 min-w-0 w-full overflow-hidden bg-gray-100 dark:bg-gray-900">
      <div class="h-full w-full min-h-0 min-w-0 overflow-y-auto overflow-x-hidden">
        <table class="w-full text-sm text-left text-gray-700 dark:text-gray-200">
          <thead class="relative">
            <tr class="bg-gray-50 dark:bg-gray-800 z-1 text-xs text-gray-500 uppercase sticky top-0">
              <th class="px-2 py-2">User</th>
              <th class="px-2 py-2">Avatar</th>
              <th class="px-2 py-2">Perf</th>
              <th class="px-2 py-2 text-right">Acc Age</th>
              <th class="px-2 py-2 text-right">Join</th>
              <th class="px-2 py-2 text-right">Leave</th>
              { /* TODO: advisory text column? maybe width-dependent? */ }
              <th class="px-2 py-2 text-right">{ /* Icons */ }</th>
            </tr>
          </thead>
          <tbody class="">
            {/* TODO: smart sort: user-specific advisories, avatar advisories, group membership advisories. Within each category, users should be sorted by most recent leave or join first */}
            {userList
            // TODO: sort by importance first, then by time
            .map((u, idx) => (
              <tr
                key={u.id}
                class={(idx % 2 === 0 ? 'bg-white dark:bg-transparent' : 'bg-gray-50 dark:bg-gray-800/30') + ' h-[32px] hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer active:bg-gray-50 dark:active:bg-black/50'}
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
                      // Creates a VRCMRD ticket, not a VRChat ticket.
                      text: 'Create moderation ticket'
                    }),
                    await menu.MenuItem.new({
                      text: 'View profile in browser'
                    }),
                    await menu.Submenu.new({
                      text: 'Suppress advisory for user',
                      items: [
                        await menu.MenuItem.new({ text: 'Watchlist 1' }),
                        await menu.MenuItem.new({ text: 'Watched avatar' }),
                      ]
                    }),
                    await menu.MenuItem.new({
                      text: 'Copy user ID'
                    }),
                  ]});
                  cm.popup();
                }}
                >
                <td class="px-2 py-1 align-middle overflow-hidden flex-grow">
                  <div class="font-medium text-sm max-w-[24ch]">{u.username}</div>
                </td>
                <td class="px-2 py-1 align-middle overflow-hidden flex-grow">
                  <div class="text-xs text-gray-500 max-w-[24ch]">{u.avatarName}</div>
                </td>
                <td class="px-2 py-1 align-middle font-semibold">{/* TODO: performance rank icons */ u.perfRank}</td>
                <td class="px-2 py-1 align-middle text-xs text-gray-500 text-right">{u.accountAge}</td>
                <td class="px-2 py-1 align-middle text-xs text-right">{new Date(u.joinTime*1000).toLocaleTimeString()}</td>
                <td class="px-2 py-1 align-middle text-xs text-right">{u.leaveTime !== null ? new Date(u.leaveTime*1000).toLocaleTimeString() : ''}</td>
                <td class="px-2 py-1 align-middle overflow-hidden flex-grow">
                  <div class="flex items-center justify-end gap-2">
                  {u.advisories && (
                    <div class="tooltip right" aria-hidden>
                      { /* TODO: change color/icon for highest level advisory */ }
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
                      {u.platform === 'pc' && <MonitorIcon className='text-blue-400' />}
                      {u.platform === 'android' && <AndroidIcon className='text-green-400' />}
                      {u.platform === 'ios' && <AppleIcon />}
                      {u.platform === null && <span class="text-xs text-gray-400"><HelpOutlineIcon /></span>}
                      <div class="tooltip-tip">Platform: {u.platform?.toUpperCase()}</div>
                  </div>
                  </div>
                </td>
              </tr>
            ))}
            </tbody>
        </table>
      </div>
    </div>
  );
}
