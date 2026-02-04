import "../index.css";
import { useState } from "preact/hooks";
import { useOverlayScrollbars } from "../components/OverlayScrollbarsHook";
import { render } from "preact";
import PlusIcon from "mdi-preact/PlusIcon";
import { Tooltip } from 'react-tooltip'
import AdvisoryList from "../advisories/list";
import { invoke } from "@tauri-apps/api/core";
import defaultAdvisory from "../advisories/default";
import AdvisoryEditor from "../advisories/editor";

function ManageAdvisoriesWindow() {
  useOverlayScrollbars();
  const [overlay, setOverlay] = useState<preact.VNode|null>(null);
  const [dialog, setDialog] = useState<preact.VNode|null>(null);
  return <>
    <div class="h-screen w-full overflow-y-auto select-none flex bg-gray-100 text-gray-600 dark:bg-gray-900 dark:text-gray-300">
      <div class="p-6">
        <h1 class="text-2xl font-bold mb-4">Manage Advisories</h1>
        <AdvisoryList setOverlay={setOverlay} setDialog={setDialog} />
        <button class="fixed bottom-4 right-4 mt-4 px-3 py-3 bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white rounded-full shadow-lg"
        aria-label="Add Advisory" data-tooltip-id="tooltip" data-tooltip-content="Add Advisory" 
        onClick={() => {
          // FAB to add new advisories
          const newAdvisory = defaultAdvisory();
          invoke<string>("generate_advisory_id").then((generatedId) => {
            newAdvisory.id = generatedId;
            newAdvisory.active = true;
            setOverlay?.(<AdvisoryEditor isNew advisory={newAdvisory} setOverlay={setOverlay} setDialog={setDialog} />);
          });
        }}><PlusIcon /></button>
      </div>
    </div>
    {overlay && <>
      <div id="scrim" class="h-screen w-screen overflow-hidden fixed inset-0 bg-black/50 z-40"></div>
      <div id="overlay" class="fixed inset-0 grid place-items-center z-50">
        {overlay}
      </div>
    </>}
    {dialog && <>
      <div id="dialog-scrim" class="h-screen w-screen overflow-hidden fixed inset-0 bg-black/50 z-50"></div>
      <div id="dialog-overlay" class="fixed inset-0 grid place-items-center z-60">
        {dialog}
      </div>
    </>}
    <Tooltip id="tooltip" />
  </>;
}

render(<ManageAdvisoriesWindow />, document.getElementById("root")!);