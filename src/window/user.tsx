//import * as tauri from "@tauri-apps/api";

import "../index.css";
import { useEffect, useState } from "preact/hooks";
import { useOverlayScrollbars } from "../components/OverlayScrollbarsHook";
import { invoke } from "@tauri-apps/api/core";
import { GetUserInfoResponse, User } from "../data/users";
import { formatAccountAge } from "../components/UserTable";
import { render } from "preact/compat";
import * as vrc from "vrchat";
import AlertIcon from "mdi-preact/AlertOutlineIcon";
import InfoIcon from "mdi-preact/InformationOutlineIcon";
import ErrorIcon from "mdi-preact/AlertIcon";
import StopIcon from "mdi-preact/AlertOctagonIcon";
import { listen } from "@tauri-apps/api/event";

export default function UserDetailsWindowContents() {
  const userId = window.location.hash.substring(1);
  const [userInfo, setUserInfo] = useState<GetUserInfoResponse|null>(null);
  const [userGroups, setUserGroups] = useState<vrc.LimitedUserGroups[]|null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  useEffect(() => {
    setLoading(true);
    invoke<GetUserInfoResponse>("get_user_info", { userId })
      .then((user) => {
        setUserInfo(user);
      })
      .catch((err) => {
        console.error("Failed to fetch user info:", err);
        setUserInfo(null);
      })
      .finally(() => {
        setLoading(false);
      });
    invoke("get_all_groups", { userId })
      .then((groups) => {
        setUserGroups(groups as vrc.LimitedUserGroups[]);
      })
      .catch((err) => {
        console.error("Failed to fetch user groups:", err);
        setUserGroups(null);
      });
    const listener = listen("vrcmrd:update-user", (event) => {
      console.log("Received user update event:", event);
      const payload = event.payload as User;
      if (payload.id == userId) {
        setUserInfo((prev) => {
          return {
            local: payload,
            remote: prev?.remote || null,
          };
        });
      } else {
        console.log("User ID does not match, ignoring update.");
      }
    });
    return () => {
      listener.then((unlisten) => unlisten());
    };
  }, [userId]);
  useOverlayScrollbars();
  if (loading && !userInfo) {
    return <div className="h-screen select-none flex items-center justify-center bg-gray-100 text-gray-600 dark:bg-gray-900 dark:text-gray-300">
      <p>Loading user info...</p>
    </div>;
  }
  if (!userInfo?.local) {
    return <div className="h-screen select-none flex items-center justify-center bg-gray-100 text-gray-600 dark:bg-gray-900 dark:text-gray-300">
      <p>Failed to load user info.</p>
    </div>;
  }
  return <div className="h-screen w-full overflow-y-auto select-none flex bg-gray-100 text-gray-600 dark:bg-gray-900 dark:text-gray-300 p-6">
    <h2 className="text-2xl font-bold mb-4">{userInfo.local!.username || userInfo.remote?.displayName}</h2>
    <div className="space-y-2">
      {/* TODO: get the full user info object from the API */}
      <p><strong>ID:</strong> {userInfo.local!.id || userInfo.remote?.id}</p>
      {/* <p><strong>Bio:</strong> {userInfo.bio || "No bio set."}</p> */}
      <p><strong>Bio:</strong> <blockquote className="italic text-gray-500 dark:text-gray-400">{userInfo.remote?.bio || "No bio set."}</blockquote></p>
      <p><strong>Platform:</strong> {userInfo.local!.platform}</p>
      {userInfo.local!.accountCreated && <p><strong>Account Created:</strong> {new Date(userInfo.local!.accountCreated * 1000).toLocaleDateString()} <em>{formatAccountAge(userInfo.local!.accountCreated)} ago</em></p>}
      <p><strong>Trust Rank:</strong> {userInfo.local!.trustRank}</p>
      {/* <p><strong>Representing Group:</strong> {userInfo.remote.representingGroup?.name}</p> */}
    </div>
    <div className="mt-4 p-2">
      <h3 className="font-semibold mb-2">Advisories</h3>
      {userInfo.local!.advisories.length > 0 && <ul className="list-disc list-inside">
          {userInfo.local!.advisories.sort((a, b) => b.level - a.level).map((adv, idx) => (
            <li key={idx}>{{
              0: <InfoIcon className="inline align-middle w-4 h-4 text-blue-400 mr-1 mb-1" />,
              1: <AlertIcon className="inline align-middle w-4 h-4 text-yellow-400 mr-1 mb-1" />,
              2: <AlertIcon className="inline align-middle w-4 h-4 text-orange-400 mr-1 mb-1" />,
              3: <ErrorIcon className="inline align-middle w-4 h-4 text-red-400 mr-1 mb-1" />,
              4: <StopIcon className="inline align-middle w-4 h-4 text-red-400 mr-1 mb-1" />,
            }[adv.level]}
            {adv.message}{adv.relevantGroupId && <>{` (Group: `}<a href={`https://vrchat.com/home/group/${adv.relevantGroupId}`} target="_blank" rel="noopener noreferrer">{adv.relevantGroupId}</a>{`)`}</>}</li>
          ))}
        </ul>}
    </div>

    <div className="mt-4 p-2">
      <h3 className="font-semibold mb-2">Group Memberships</h3>
      {userGroups && userGroups.length > 0 ? <div class="grid grid-cols-3 md:grid-cols-4 lg:grid-cols-5  gap-4">
        {userGroups.filter((v) => v != null).map((group, idx) => (
          <a href={`https://vrchat.com/home/group/${group.groupId}`} target="_blank" rel="noopener noreferrer" key={idx} class="border border-gray-300 dark:border-gray-700 rounded bg-white dark:bg-gray-800 flex flex-col hover:bg-gray-50 dark:hover:bg-gray-700">
            <img src={group.bannerUrl || "https://vrchat.com/images/group_placeholder.png"} alt="Group Banner" class="rounded-t" />
            {/* TODO: show group banners; make it a grid */}
            <span class="mx-2 my-1 font-medium text-blue-600 dark:text-blue-400 hover:underline">{group.name ?? (group.shortCode??"(unknown)")+"."+(group.discriminator??"0000")}</span>
          </a>
        ))}
      </div> : <p>Not a public member of any group.</p>}
    </div>
  </div>;
}

render(<UserDetailsWindowContents />, document.getElementById("root")!);