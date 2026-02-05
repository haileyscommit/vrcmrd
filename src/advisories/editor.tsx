import { Advisory } from "@app/bindings/Advisory";
import { invoke } from "@tauri-apps/api/core";
import CloseIcon from "mdi-preact/CloseIcon";
import DeleteIcon from "mdi-preact/DeleteIcon";
import AlertIcon from "mdi-preact/AlertOutlineIcon";
import InfoOutlineIcon from "mdi-preact/InformationOutlineIcon";
import InfoFilledIcon from "mdi-preact/InformationIcon";
import ErrorIcon from "mdi-preact/AlertIcon";
import StopIcon from "mdi-preact/AlertOctagonIcon";
import { useState } from "preact/compat";
import ConditionEditor, { NestedConditionTypes } from "./condition";
import Dropdown from "../components/Dropdown";
import PlusIcon from "mdi-preact/PlusIcon";

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
        <Dropdown items={[
          { active: level as any === 0, set: () => setLevel(0 as any), label: <div class="flex flex-row justify-start gap-1 items-center"><InfoOutlineIcon class="inline-block align-middle w-4 h-4 text-black dark:text-white" />None</div>, description: <>To add icons and call-outs for moderators or VIPs. Users with advisories at this level are not prioritized in the list.</> },
          { active: level as any === 1, set: () => setLevel(1 as any), label: <div class="flex flex-row justify-start gap-1 items-center"><InfoFilledIcon class="inline-block align-middle w-4 h-4 text-blue-400" />Low</div>, description: <>For users who are slightly more likely to cause problems (i.e. new users)</> },
          { active: level as any === 2, set: () => setLevel(2 as any), label: <div class="flex flex-row justify-start gap-1 items-center"><AlertIcon class="inline-block align-middle w-4 h-4 text-yellow-400" />Medium</div>, description: <>Likely to need action or attention (i.e. an unusual log event)</> },
          { active: level as any === 3, set: () => setLevel(3 as any), label: <div class="flex flex-row justify-start gap-1 items-center"><ErrorIcon class="inline-block align-middle w-4 h-4 text-orange-400" />High</div>, description: <>Known or likely offenders (i.e. harasser groups or Nuisance rank)</> },
          { active: level as any === 4, set: () => setLevel(4 as any), label: <div class="flex flex-row justify-start gap-1 items-center"><StopIcon class="inline-block align-middle w-4 h-4 text-red-400" />Maximum</div>, description: <>Known offenders (i.e. crashers). <strong>Cuts off any existing TTS notifications</strong> if TTS is on, to play this one instead.</> },          
        ]} />
        <p class="mb-2 text-sm text-gray-600 dark:text-gray-400">Determines the severity of the advisory. Higher levels may trigger more noticeable notifications, and they will show up higher in the list.</p>
      </div>
      <div class="my-4">
        <label class="block mb-2 font-bold" for="advisory-message-template-input">Message Template:</label>
        <textarea id="advisory-message-template-input" class="w-full p-2 border border-gray-300 dark:border-gray-700 rounded" value={messageTemplate} onInput={(e) => setMessageTemplate((e.target as HTMLTextAreaElement).value)} rows={4} />
        {/* Wrapping flex-row of chips that add templates when pressed */}
        <div class="mt-2 flex flex-row gap-2 flex-wrap">
          {NestedConditionTypes(condition).includes("UsernameContains") && <button class="px-2 py-1 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded-full text-sm hover:bg-gray-300 dark:hover:bg-gray-600" onClick={() => {
            const insert = "{{:username:}}";
            setMessageTemplate(messageTemplate + insert);
          }}><PlusIcon class="inline align-middle w-4 h-4 mr-1 mb-1" />Username</button>}
          {NestedConditionTypes(condition).includes("AccountAgeAtMostDays") && <button class="px-2 py-1 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded-full text-sm hover:bg-gray-300 dark:hover:bg-gray-600" onClick={() => {
            const insert = "{{:account_age_days:}}";
            setMessageTemplate(messageTemplate + insert);
          }}><PlusIcon class="inline align-middle w-4 h-4 mr-1 mb-1" />Account Age (days)</button>}
          {NestedConditionTypes(condition).includes("IsGroupMember") && <button class="px-2 py-1 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded-full text-sm hover:bg-gray-300 dark:hover:bg-gray-600" onClick={() => {
            const insert = "{{:group_name:}}";
            setMessageTemplate(messageTemplate + insert);
          }}><PlusIcon class="inline align-middle w-4 h-4 mr-1 mb-1" />Group Name</button>}
        </div>
        <p class="mb-2 text-sm text-gray-600 dark:text-gray-400">The message that will be shown or spoken when this advisory is applied. You can use variables like <code>{'{{:variable||default:}}'}</code> to include specific context.</p>
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