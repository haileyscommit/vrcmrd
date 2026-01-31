//import * as tauri from "@tauri-apps/api";

import "../index.css";
import { useEffect, useState } from "preact/hooks";
import { useOverlayScrollbars } from "../components/OverlayScrollbarsHook";
import { invoke } from "@tauri-apps/api/core";
import { GetUserInfoResponse } from "../data/users";
import { formatAccountAge } from "../components/UserTable";
import { render } from "preact/compat";

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
  if (loading) {
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
        {userInfo.local!.accountCreated && <p><strong>Account Created:</strong> {new Date(userInfo.local!.accountCreated * 1000).toLocaleDateString()} <em>{formatAccountAge(userInfo.local!.accountCreated)} ago</em></p>}
        <p><strong>Trust Rank:</strong> {userInfo.local!.trustRank}</p>
      </div>
      {userInfo.local!.advisories.length > 0 && <div className="mt-4 p-2 rounded">
        <h3 className="font-semibold mb-2">Advisories</h3>
        <ul className="list-disc list-inside">
          {userInfo.local!.advisories.sort((a, b) => b.level - a.level).map((adv, idx) => (
            <li key={idx}>{adv.message}{adv.relevantGroupId && <>{` (Group: `}<a href={`https://vrchat.com/home/group/${adv.relevantGroupId}`} target="_blank" rel="noopener noreferrer">{adv.relevantGroupId}</a>{`)`}</>}</li>
          ))}
        </ul>
      </div>}
    </div>
  </div>;
}

render(<UserDetailsWindowContents />, document.getElementById("root")!);