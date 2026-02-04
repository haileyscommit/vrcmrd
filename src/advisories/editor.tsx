import { Advisory } from "@app/bindings/Advisory";
import { Menu, MenuButton, MenuItem, MenuItems } from "@headlessui/react";
import { invoke } from "@tauri-apps/api/core";
import CloseIcon from "mdi-preact/CloseIcon";
import DeleteIcon from "mdi-preact/DeleteIcon";
import DownArrowIcon from "mdi-preact/MenuDownIcon";
import { useState } from "preact/compat";
import ConditionEditor from "./condition";

export default function AdvisoryEditor({ advisory, isNew, setOverlay, setDialog }: { 
  advisory: Advisory,
  isNew?: boolean,
  setOverlay?: (overlay: preact.VNode|null) => void,
   setDialog?: (dialog: preact.VNode|null) => void
}) {
  const [name, setName] = useState(advisory.name);
  const [messageTemplate, setMessageTemplate] = useState(advisory.message_template);
  const [level, setLevel] = useState(advisory.level);
  const [active, setActive] = useState(advisory.active);
  const [sendNotification, setSendNotification] = useState(advisory.send_notification);
  const [sendTts, setSendTts] = useState(advisory.send_tts);
  const [condition, setCondition] = useState(advisory.condition);
  return <div class="select-none h-full w-full p-2"><div class="max-w-3xl ml-auto flex flex-col bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 h-full w-full px-6 py-2 rounded shadow-lg">
    <div class="w-full flex flex-row mb-4 gap-2 items-center">
      <button class="inline-block bg-transparent hover:bg-black/20 hover:dark:bg-white/20 text-white transition rounded-full p-2 m-2" onClick={() => setOverlay?.(null)} aria-label="Close"><CloseIcon /></button>
      <span class="text-2xl font-bold">Manage "{advisory.name}"</span>
      <span class="flex-grow"></span>
      <button class="inline-block bg-transparent hover:bg-black/20 hover:dark:bg-white/20 text-white hover:text-red-400 transition rounded-full p-2 m-2" onClick={() => {
        setDialog?.(<div class="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 p-6 rounded shadow-lg">
          <h2 class="text-xl font-bold mb-4">Delete Advisory</h2>
          <p>Are you sure you want to delete this advisory?</p>
          <button class="mt-4 px-4 py-2 hover:bg-gray-200 dark:hover:bg-gray-700 text-white rounded" onClick={() => setDialog?.(null)}>Cancel</button>
          <button class="ml-4 mt-4 px-4 py-2 bg-red-400 text-white rounded" onClick={() => {
            invoke("remove_advisory", {advisoryId: advisory.id}).then(() => {
              setDialog?.(null);
              setOverlay?.(null);
            });
          }}>Delete</button>
        </div>);
      }}><DeleteIcon /></button>
    </div>
    <div class="overflow-y-auto h-[calc(100vh-7rem)]" id="advisory-overlay-content">
      {/* Active: use switch */}
      <div class="my-4 flex items-center gap-2">
        <label class="font-bold" for="advisory-active-input">Active:</label>
        <input id="advisory-active-input" type="checkbox" checked={active} onChange={(e) => setActive((e.target as HTMLInputElement).checked)} />
      </div>
      <p>Advisory ID: <span class="select-all font-mono">{advisory.id}</span></p>
      <div class="my-4">
        <label class="block mb-2 font-bold" for="advisory-name-input">Advisory Name:</label>
        <input id="advisory-name-input" type="text" class="w-full p-2 border border-gray-300 dark:border-gray-700 rounded" value={name} onInput={(e) => setName((e.target as HTMLInputElement).value)} />
      </div>
      <ConditionEditor condition={condition} setCondition={setCondition} />
      <div class="my-4 relative">
        <label class="block mb-2 font-bold" for="advisory-level-input">Advisory Level:</label>
        <Menu>
          <MenuButton id="advisory-level-input" class="w-full p-2 border border-gray-300 dark:border-gray-700 rounded text-left flex justify-between items-center">
            {level as any === 0 && "None"}
            {level as any === 1 && "Low"}
            {level as any === 2 && "Medium"}
            {level as any === 3 && "High"}
            {level as any === 4 && "Maximum"}
            <DownArrowIcon />
          </MenuButton>
          <MenuItems class="my-1 w-full px-4 absolute">
            <div class="z-55 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded shadow-lg p-2 space-y-2 overflow-y-auto max-h-60 text-black dark:text-white">
              <MenuItem>
                <button class={`w-full text-left p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg ${level === 0 as any && "bg-gray-100 dark:bg-gray-700"}`} onClick={() => setLevel(0 as any)}>None<br /><span class="text-sm text-black/70 dark:text-white/70">To add icons and call-outs for moderators or VIPs. Users with advisories at this level are not prioritized in the list.</span></button>
              </MenuItem>
              <MenuItem>
                <button class={`w-full text-left p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg ${level === 1 as any && "bg-gray-100 dark:bg-gray-700"}`} onClick={() => setLevel(1 as any)}>Low<br /><span class="text-sm text-black/70 dark:text-white/70">For users who are slightly more likely to cause problems (i.e. new users)</span></button>
              </MenuItem>
              <MenuItem>
                <button class={`w-full text-left p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg ${level === 2 as any && "bg-gray-100 dark:bg-gray-700"}`} onClick={() => setLevel(2 as any)}>Medium<br /><span class="text-sm text-black/70 dark:text-white/70">Likely to need action or attention (i.e. an unusual log event)</span></button>
              </MenuItem>
              <MenuItem>
                <button class={`w-full text-left p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg ${level === 3 as any && "bg-gray-100 dark:bg-gray-700"}`} onClick={() => setLevel(3 as any)}>High<br /><span class="text-sm text-black/70 dark:text-white/70">Known or likely offenders (i.e. harasser groups or Nuisance rank)</span></button>
              </MenuItem>
              <MenuItem>
                <button class={`w-full text-left p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg ${level === 4 as any && "bg-gray-100 dark:bg-gray-700"}`} onClick={() => setLevel(4 as any)}>Maximum<br /><span class="text-sm text-black/70 dark:text-white/70">Known offenders (i.e. crashers). <strong>Cuts off any existing TTS notifications</strong> if TTS is on, to play this one instead.</span></button>
              </MenuItem>
            </div>
          </MenuItems>
        </Menu>
        <p class="mb-2 text-sm text-gray-600 dark:text-gray-400">Determines the severity of the advisory. Higher levels may trigger more noticeable notifications, and they will show up higher in the list.</p>
      </div>
      <div class="my-4">
        <label class="block mb-2 font-bold" for="advisory-message-template-input">Message Template:</label>
        <textarea id="advisory-message-template-input" class="w-full p-2 border border-gray-300 dark:border-gray-700 rounded" value={messageTemplate} onInput={(e) => setMessageTemplate((e.target as HTMLTextAreaElement).value)} rows={4} />
      </div>
      <div class="my-4 flex items-center gap-2">
        <input id="send-notification-input" type="checkbox" checked={sendNotification} onChange={(e) => setSendNotification((e.target as HTMLInputElement).checked)} />
        <label class="font-bold" for="send-notification-input">Send desktop, XSOverlay, or OVR Toolkit notification</label>
      </div>
      <div class="my-4 flex items-center gap-2">
        <input id="send-tts-input" type="checkbox" checked={sendTts} onChange={(e) => setSendTts((e.target as HTMLInputElement).checked)} />
        <label class="font-bold" for="send-tts-input">Speak message via TTS</label>
      </div>
      <div class="mt-6 flex flex-row gap-4 justify-end">
        {/* <button class="px-4 py-2 hover:bg-gray-200 dark:hover:bg-gray-700 text-white rounded" onClick={() => setOverlay?.(null)}>Cancel</button>*/}
        <button class="px-4 py-2 bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white rounded" onClick={() => {
          const updatedAdvisory: Advisory = { ...advisory, active, name, message_template: messageTemplate, level, send_notification: sendNotification, send_tts: sendTts, condition };
          invoke(isNew ? "add_advisory" : "update_advisory", { advisory: updatedAdvisory }).then(() => {
            setOverlay?.(null);
          });
        }}>Save Changes</button>
      </div>
    </div>
  </div></div>;
}