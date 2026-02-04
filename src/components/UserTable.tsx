/// <reference types="vite-plugin-svgr/client" />
import { getHighestAdvisoryLevel, TrustRank, User } from '../data/users';
import AlertIcon from "mdi-preact/AlertIcon";
import AndroidIcon from "mdi-preact/AndroidIcon";
import AppleIcon from "mdi-preact/AppleIcon";
import MonitorIcon from "mdi-preact/MonitorIcon";
import HelpOutlineIcon from "mdi-preact/HelpCircleOutlineIcon";
import InfoIcon from "mdi-preact/InformationOutlineIcon";
import ErrorIcon from "mdi-preact/AlertCircleIcon";
import StopIcon from "mdi-preact/AlertOctagonIcon";
import { useEffect, useState } from 'preact/hooks';
import { listen } from '@tauri-apps/api/event';
import { menu } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { Tooltip } from 'react-tooltip';

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
    //var currentInstanceId: string | null = null;
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
    const instanceUnlisten = listen('vrcmrd:instance', (_) => {
      setUserList([]);
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
    const updateAllUnlisten = listen('vrcmrd:users-updated', (_) => {
      softRefreshList();
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
      updateAllUnlisten.then(unlisten => unlisten());
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
                class={(idx % 2 === 0 ? 'bg-white dark:bg-transparent' : 'bg-gray-50 dark:bg-gray-800/30') + (u.leaveTime ? ' opacity-75' : '') + ' h-[32px] hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer active:bg-gray-50 dark:active:bg-black/50'}
                onClick={() => invoke("show_user_details", {user: u.id})}
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
                <td class="px-2 align-middle font-semibold">
                  <div class="tooltip" aria-hidden data-tooltip-id="tooltip" data-tooltip-content={u.perfRank}>
                    {u.perfRank ? <img src={{
                      "Excellent": "/assets/perf/excellent.png",
                      "Good": "/assets/perf/good.png",
                      "Medium": "/assets/perf/medium.png",
                      "Poor": "/assets/perf/poor.png",
                      "VeryPoor": "/assets/perf/very-poor.png",
                    }[u.perfRank]} class="w-5 h-5" /> : "?"}
                    <div class="tooltip-tip">{u.perfRank}</div>
                  </div>
                </td>
                <td class="px-2 py-1 align-middle text-xs text-gray-500 text-right">{formatAccountAge(u.accountCreated)}</td>
                <td class="px-2 py-1 align-middle text-xs text-right">{new Date(u.joinTime*1000).toLocaleTimeString()}</td>
                <td class="px-2 py-1 align-middle text-xs text-right">{u.leaveTime !== null ? new Date(u.leaveTime*1000).toLocaleTimeString() : ''}</td>
                <td class="px-2 py-1 align-middle overflow-hidden flex-grow">
                  <div class="flex items-center justify-end gap-2">
                  {u.advisories.length > 0 && (
                    <div class="tooltip left" aria-hidden data-tooltip-id="tooltip" data-tooltip-content={`${u.advisories.length} advisories`}>
                      { /* TODO: change color/icon for highest level advisory */ }
                      {{
                        0: <InfoIcon class="w-4 h-4 text-black dark:text-white" />,
                        1: <AlertIcon class="w-4 h-4 text-yellow-400" />,
                        2: <AlertIcon class="w-4 h-4 text-orange-400" />,
                        3: <ErrorIcon class="w-4 h-4 text-red-400" />,
                        4: <StopIcon class="w-4 h-4 text-red-400" />,
                      }[getHighestAdvisoryLevel(u.advisories) ?? 0]}
                      {/* Show advisory message if only one at highest level */}
                      <div class="tooltip-tip">{u.advisories.length} advisories</div>
                    </div>
                  )}

                  {/* TODO: merge with Trust Rank
                  {u.ageVerified && !u.trustRank && (
                      <div class="tooltip right" aria-hidden>
                      <CardAccountDetailsIcon />
                      <div class="tooltip-tip">Age verified</div>
                      </div>
                  )} */}

                  {u.trustRank && (
                    <div class="tooltip" aria-hidden data-tooltip-id="tooltip" data-tooltip-content={trustRankLabel(u.trustRank, u.ageVerified)}>
                      {/* TODO: hide Nuisance and turn it into an advisory, or use a different icon */}
                      <svg viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg" class={"w-4 h-4" + {
                          "Nuisance": " text-gray-400",
                          "Visitor": " text-black dark:text-white",
                          "NewUser": " text-blue-400",
                          "User": " text-green-400",
                          "KnownUser": " text-orange-400",
                          "TrustedUser": " text-purple-400",
                          "Admin": " text-red-400"
                        }[u.trustRank]}>
                        <use href={u.ageVerified ? '/assets/trust-verified.svg' : '/assets/trust.svg'} aria-label={trustRankLabel(u.trustRank, u.ageVerified)} />
                      </svg>
                      <div class="tooltip-tip">{trustRankLabel(u.trustRank, u.ageVerified)}</div>
                    </div>
                  )}

                  <div class="tooltip" aria-hidden data-tooltip-id="tooltip" data-tooltip-content={`Platform: ${u.platform?.toUpperCase()}`}>
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
      <Tooltip id="tooltip" />
    </div>
  );
}

export function formatAccountAge(accountCreated: number | null): string {
  if (accountCreated === null) return '';
  const now = Date.now() / 1000;
  const ageSeconds = now - accountCreated;
  const ageDays = Math.floor(ageSeconds / (60 * 60 * 24));
  // TODO: consider account "ages" in the future (make them negative)
  if (ageDays >= 365) {
    const years = Math.floor(ageDays / 365);
    return `${years}y`;
  } else if (ageDays >= 30) {
    const months = Math.floor(ageDays / 30);
    return `${months}mo`;
  } else if (ageDays >= 1) {
    return `${ageDays}d`;
  } else if (ageSeconds >= 3600) {
    return `${Math.floor(ageSeconds / 3600)}h`;
  } else if (ageSeconds >= 60) {
    return `${Math.floor(ageSeconds / 60)}m`;
  } else {
    return `${Math.floor(ageSeconds)}s`;
  }
}

export function trustRankLabel(rank: TrustRank, ageVerified?: boolean): string {
  const base = {
    "Nuisance": "Nuisance",
    "Visitor": "Visitor",
    "NewUser": "New User",
    "User": "User",
    "KnownUser": "Known User",
    "TrustedUser": "Trusted User",
    "Admin": "VRChat Staff"
  }[rank] || "Unknown";
  return ageVerified ? `${base} (Age Verified)` : base;
}