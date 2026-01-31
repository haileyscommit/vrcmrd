import { invoke } from "@tauri-apps/api/core";
import { useState } from "preact/hooks";

export default function NetworkingSettingsPage({ loading, setLoading }: { loading: boolean; setLoading: (v: boolean) => void }) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  return (
    <div className={`space-y-4 ${loading ? "opacity-50 pointer-events-none cursor-wait" : ""}`}>
      <h2 className="text-lg font-medium text-gray-900 dark:text-gray-100">VRChat Credentials</h2>
      <p className="text-sm text-gray-600 dark:text-gray-300">We need these to access the VRChat API, which is used to get information about other users in the instance.</p>
      <div className="space-y-2">
        <label className="block text-xs text-gray-600 dark:text-gray-400">Username</label>
        <input disabled={loading} className="w-full rounded-md bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 px-3 py-2 border border-gray-300 dark:border-gray-700" placeholder="VRChat Username" value={username} onInput={(e) => setUsername(e.currentTarget.value)} />
      </div>
      <div className="space-y-2">
        <label className="block text-xs text-gray-600 dark:text-gray-400">Password</label>
        <input disabled={loading} type="password" className="w-full rounded-md bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 px-3 py-2 border border-gray-300 dark:border-gray-700" placeholder="VRChat Password" value={password} onInput={(e) => setPassword(e.currentTarget.value)} />
      </div>
      <div className="flex flex-row gap-2">
        <button className="mt-4 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-md text-sm font-medium" onClick={async () => {
          setLoading(true);
          try {
            await invoke("update_credentials", { username, password });
            setUsername("");
            setPassword("");
            alert("You are now logged in!");
          } catch (e) {
            console.error("Failed to update credentials:", e);
            alert("Could not log in. Please check your username and password, and try again.");
          } finally {
            setLoading(false);
          }
        }} disabled={loading}>{loading ? "Saving..." : "Save & Login"}</button>
        <span className="flex-1"></span>
        <button className="mt-4 px-4 py-2 text-red-600 hover:bg-red-500/20 rounded-md text-sm font-medium" onClick={async () => {
          setUsername("");
          setPassword("");
          setLoading(true);
          try {
            await invoke("logout", {});
            alert("You have been logged out.");
          } catch (e) {
            console.error("Failed to log out:", e);
            alert("Could not log out. Please try again.");
          } finally {
            setLoading(false);
          }
        }} disabled={loading}>Log Out</button>
      </div>
      {/* TODO: Iroh relay setting, relay mode if that's possible to set (i.e. always relay, direct connect where possible, force direct connect) */}
      {/* TODO: your own Discord username or ID, to use with Discord ticket integration */}
    </div>
  );
}