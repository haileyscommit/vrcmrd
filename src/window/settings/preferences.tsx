import { invoke } from "@tauri-apps/api/core"
import Dropdown from "../../components/Dropdown"
import { useEffect, useState } from "preact/hooks"
import { listen } from "@tauri-apps/api/event";
import { Checkbox } from "@headlessui/react";
import CheckIcon from "mdi-preact/CheckIcon";

export default function PreferencesPage({ loading }: { loading: boolean }) {
  const [notificationPreference, setNotificationPreference] = useState("1");
  const [ttsPreference, setTtsPreference] = useState("1");
  function set(key: string, value: string) {
    invoke("update_config", { key, value }).catch((e) => {
      console.error(`Failed to set config ${key} to ${value}:`, e);
      alert(`Could not save preference ${key}. Please try again.`);
    });
  }
  useEffect(() => {
    invoke("get_config", { key: "notification_preference" }).then((value) => {
      if (typeof value === "string") {
        setNotificationPreference(value);
      }
    });
    invoke("get_config", { key: "tts_preference" }).then((value) => {
      if (typeof value === "string") {
        setTtsPreference(value);
      }
    });
    let updated = listen("vrcmrd:config_updated", (event) => {
      const { key, value } = event.payload as { key: string, value: string };
      if (key === "notification_preference") {
        setNotificationPreference(value);
      } else if (key === "tts_preference") {
        setTtsPreference(value);
      }
    });
    return () => {
      updated.then((unlisten) => unlisten());
    };
  }, []);
  return <div className={`space-y-4 ${loading ? "opacity-50 pointer-events-none cursor-wait" : ""}`}>
    <h2 className="text-lg font-medium text-gray-900 dark:text-gray-100">Preferences</h2>
    <div className="space-y-2">
      <label className="block text-xs text-gray-600 dark:text-gray-400">Notification Preference</label>
      <Dropdown items={[
        { active: notificationPreference === "0", set: () => set("notification_preference", "0"), label: <>Don't show me notifications</> },
        { active: notificationPreference === "1", set: () => set("notification_preference", "1"), label: <>Show me notifications for my own advisories</> },
        { active: notificationPreference === "2", set: () => set("notification_preference", "2"), label: <>Show me notifications for all notifying advisories</> },
        // { active: notificationPreference === "3", set: () => set("notification_preference", "3"), label: <>Show me notifications for ALL advisories</> }
      ]} />
    </div>
    <div className="space-y-2">
      <label className="block text-xs text-gray-600 dark:text-gray-400">Text-to-Speech Preference</label>
      <Dropdown items={[
        { active: ttsPreference === "0", set: () => set("tts_preference", "0"), label: <>Don't read out any advisories</> },
        { active: ttsPreference === "1", set: () => set("tts_preference", "1"), label: <>Read out my own advisories</> },
        { active: ttsPreference === "2", set: () => set("tts_preference", "2"), label: <>Read out all notifying advisories</> },
        // { active: ttsPreference === "3", set: () => set("tts_preference", "3"), label: <>Read out ALL advisories</> }
      ]} />
    </div>
    <div className="space-y-2">
      <label className="block text-xs text-gray-600 dark:text-gray-400">Age verification in user list</label>
      <Dropdown items={[
        { active: false, set: () => {}, label: <>Don't show age verification</> },
        { active: true, set: () => {}, label: <>Show along with trust rank where possible</> },
        { active: false, set: () => {}, label: <>Show separately if verified</> },
        { active: false, set: () => {}, label: <>Always show verified icon separately</> },
      ]} />
    </div>
    <CheckboxPreference label="Show platform in user list" configKey="show_platform" />
  </div>
}

export function CheckboxPreference({ label, configKey } : { label: preact.VNode | string, configKey: string }) {
  const [enabled, setEnabled] = useState(false);
  function set(value: boolean) {
    invoke("update_config", { key: configKey, value: value ? "1" : "0" }).catch((e) => {
      console.error(`Failed to set config ${configKey} to ${value}:`, e);
      alert(`Could not save preference ${configKey}. Please try again.`);
    });
  }
  useEffect(() => {
    invoke("get_config", { key: configKey }).then((value) => {
      if (typeof value === "string") {
        setEnabled(value === "1");
      }
    });
    const updated = listen("vrcmrd:config_updated", (event) => {
      const { key: updatedKey, value } = event.payload as { key: string, value: string };
      if (updatedKey === configKey) {
        setEnabled(value === "1");
      }
    });
    return () => {
      updated.then((unlisten) => unlisten());
    };
  }, [configKey]);
  return <div className="flex flex-row items-center gap-4">
    <Checkbox
      checked={enabled}
      onChange={(value) => set(value)}
      className="inline-block align-middle group size-6 rounded bg-white/10 p-1 ring-1 ring-white/15 ring-inset focus:not-data-focus:outline-none data-checked:bg-white data-focus:outline data-focus:outline-offset-2 data-focus:outline-white"
    >
      <CheckIcon className="hidden size-4 fill-black group-data-checked:block" />
    </Checkbox>
    <label className="inline-block align-middle text-sm text-gray-800 dark:text-gray-200">{label}</label>
  </div>;
}
