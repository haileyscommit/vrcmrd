import { h } from 'preact';
import { users } from '../data/users';
import AlertIcon from "mdi-preact/AlertIcon";
import CardAccountDetailsIcon from "mdi-preact/CardAccountDetailsIcon";
import AndroidIcon from "mdi-preact/AndroidIcon";
import AppleIcon from "mdi-preact/AppleIcon";
import MonitorIcon from "mdi-preact/MonitorIcon";

const Avatar = ({ name }: { name: string }) => {
  const initials = name.split(' ').map(s => s[0]).slice(0,2).join('').toUpperCase();
  // simple deterministic color by char code
  const hue = (name.charCodeAt(0) * 37) % 360;
  const style = { background: `hsl(${hue} 70% 65%)` } as any;
  return (
    <div class="avatar-sm text-white" style={style} title={name} aria-hidden>
      {initials}
    </div>
  );
};

export default function UserTable() {
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
              <th class="px-2 py-2">Flags</th>
          </tr>
          </thead>
          <tbody>
            {/* TODO: smart sort: user-specific advisories, avatar advisories, group membership advisories. Within each category, users should be sorted by most recent leave or join first */}
          {users.map((u, idx) => (
              <tr key={u.id} class={(idx % 2 === 0 ? 'bg-white' : 'bg-gray-50') + ' dark:bg-transparent hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer active:bg-gray-50 dark:active:bg-black/50'}>
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
                  <div class="flex items-center gap-2">
                  {u.advisories && (
                      <div class="tooltip" aria-hidden>
                      <AlertIcon />
                      <div class="tooltip-tip">Advisories added by user</div>
                      </div>
                  )}

                  {u.ageVerified && (
                      <div class="tooltip" aria-hidden>
                      <CardAccountDetailsIcon />
                      <div class="tooltip-tip">Age verified</div>
                      </div>
                  )}

                  <div class="tooltip" aria-hidden>
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
