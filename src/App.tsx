import UserTable from "./components/UserTable";
import Tabs from "./components/Tabs";
import Menubar from "./components/Menubar";
import { BlockingModsTable } from "./components/BlocksTable";
import { useEffect } from "preact/hooks";
import { event } from "@tauri-apps/api";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

export default function App() {
  if (import.meta.env.PROD) {
    window.addEventListener("contextmenu", e => e.preventDefault());
  }
  useEffect(() => {
    const unlisten = event.listen("vrcmrd:instance", async ev => {
      console.log("Instance changed:", ev.payload);
      const window = await getCurrentWindow();
      window.setTitle(`VRCMRD - ${ev.payload}`); // mostly a temporary thing; will be replaced with parsed instance info
      // await invoke("get_instance_id_info").then(info => {
        
      // });
      // TODO: use API to get world/group name
      // TODO: use API to get better info about the instance (instance name, etc)
    });
    return () => {
      unlisten.then(f => f());
    }
  }, []);
  useEffect(() => {
    event.emit("vrcmrd:ui-ready", {});
    const unlisten = event.listen("vrcmrd:auth-token-needed", async ev => {
      console.log("Requesting auth token from user", ev.payload);
      const label =
        ev.payload && typeof ev.payload === "object" && "requester" in ev.payload
          ? `Enter auth token for ${ev.payload.requester}`
          : `Enter auth token`;
      const token = window.prompt(label, "");
      if (token === null) {
        console.log("User cancelled auth token request", ev.payload);
        invoke("cancel_login", { });
        //await event.emit("vrcmrd:auth-token-cancelled", { payload: ev.payload });
      } else {
        console.log("Auth token provided by user");
        invoke("submit_2fa_token", { token });
        //await event.emit("vrcmrd:auth-token-provided", { token, payload: ev.payload });
      }
    });
    return () => {
      unlisten.then(f => f());
    }
  }, []);
  return (
    <div class="select-none h-screen w-screen flex flex-col items-start justify-start bg-gray-100 dark:bg-gray-900">
      <Menubar />
      <Tabs
        tabs={[
          { id: "in-world", label: "In World", content: <UserTable /> },
          {
            id: "tickets",
            label: "Tickets",
            content: (
              <div className="text-sm text-gray-600 dark:text-gray-300">
                This is where your active mod-action tickets will appear.
              </div>
            ),
          },
          {
            id: "events",
            label: "History",
            content: (
              <div className="text-sm text-gray-600 dark:text-gray-300">
                Certain logged and notifiable events will appear here, such as advisories from joining users, avatar changes, messages from the world, and more.
                You may not have access to all event types.
              </div>
            ),
          },
          { id: "blocking-mods", label: "Blocking Mods", content: <BlockingModsTable /> },
        ]}
      />
    </div>
  );
}
