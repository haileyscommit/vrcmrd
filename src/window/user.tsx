//import * as tauri from "@tauri-apps/api";

import "../index.css";
import { useEffect, useState } from "preact/hooks";
import { useOverlayScrollbars } from "../components/OverlayScrollbarsHook";
import { invoke } from "@tauri-apps/api/core";
import { GetUserInfoResponse } from "../data/users";
import { formatAccountAge } from "../components/UserTable";
import { render } from "preact/compat";
import HelpOutlineIcon from "mdi-preact/HelpCircleOutlineIcon";
import InfoIcon from "mdi-preact/InformationOutlineIcon";
import AlertIcon from "mdi-preact/AlertIcon";
import ErrorIcon from "mdi-preact/AlertCircleIcon";
import StopIcon from "mdi-preact/AlertOctagonIcon";

export default function UserDetailsWindowContents() {
  const userId = window.location.hash.substring(1);
  const [userInfo, setUserInfo] = useState<GetUserInfoResponse|null>(null);
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
    return () => {};
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
  return <div className="h-screen select-none flex bg-gray-100 text-gray-600 dark:bg-gray-900 dark:text-gray-300">
    <div className="m-4 p-4 bg-white dark:bg-gray-800 rounded shadow-sm border border-gray-200 dark:border-gray-700 w-full max-w-md">
      <h2 className="text-2xl font-bold mb-4">{userInfo.local!.username || userInfo.remote?.display_name}</h2>
      <div className="space-y-2">
        {/* TODO: get the full user info object from the API */}
        <p><strong>ID:</strong> {userInfo.local!.id || userInfo.remote?.id}</p>
        {/* <p><strong>Bio:</strong> {userInfo.bio || "No bio set."}</p> */}
        <p><strong>Bio:</strong> <blockquote className="italic text-gray-500 dark:text-gray-400">{userInfo.remote?.bio || "No bio set."}</blockquote></p>
        <p><strong>Platform:</strong> {userInfo.local!.platform}</p>
        {userInfo.local!.accountCreated && <p><strong>Account Created:</strong> {new Date(userInfo.local!.accountCreated * 1000).toLocaleDateString()} <em>{formatAccountAge(userInfo.local!.accountCreated)} ago</em></p>}
        <p><strong>Trust Rank:</strong> {userInfo.local!.trustRank}</p>
      </div>
      {userInfo.local!.advisories.length > 0 && <div className="mt-4 p-2 rounded">
        <h3 className="font-semibold mb-2">Advisories</h3>
        <ul className="list-disc list-inside">
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
        </ul>
      </div>}
      {/* Group memberships */}
    </div>
  </div>;
}

render(<UserDetailsWindowContents />, document.getElementById("root")!);