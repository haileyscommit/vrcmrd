import "../index.css";
import { useState } from "preact/hooks";
import { useOverlayScrollbars } from "../components/OverlayScrollbarsHook";
import { render } from "preact";
import CloseIcon from "mdi-preact/CloseIcon";
import PlusIcon from "mdi-preact/PlusIcon";
import { Tooltip } from 'react-tooltip'
import AdvisoryList from "../advisories/list";
import { invoke } from "@tauri-apps/api/core";
import { Advisory } from "@app/bindings/Advisory";
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
        {/* <button class="mt-4 px-4 py-2 bg-blue-600 text-white rounded" onClick={() => {
          setDialog((_) => <div class="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 p-6 rounded shadow-lg">
            <h2 class="text-xl font-bold mb-4">Advisory Overlay</h2>
            <p>This is an overlay for managing advisories.</p>
            <button class="mt-4 px-4 py-2 bg-red-600 text-white rounded" onClick={() => setDialog(null)}>Close</button>
          </div>);
        }}>Example dialog</button> */}
        {/* <button class="ml-4 mt-4 px-4 py-2 bg-blue-600 text-white rounded" onClick={() => {
          setOverlay((_) => <div class="h-full w-full p-2"><div class="flex flex-col bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 h-full w-full px-6 py-2 rounded shadow-lg">
            {/ * The actual advisory management interface will probably be wrapped like this. * /}
            <div class="w-full flex flex-row mb-4 gap-2 items-center">
              <button class="inline-block bg-transparent hover:bg-black/20 hover:dark:bg-white/20 text-white rounded-full p-2 m-2" onClick={() => setOverlay(null)} aria-label="Close"><CloseIcon /></button>
              <span class="text-2xl font-bold">Manage Advisory</span>
            </div>
            <div class="overflow-y-auto h-[calc(100vh-7rem)]" id="advisory-overlay-content">
              <p>This is an overlay for managing advisories.</p>
              {".".repeat(100).split("").map((_,i) => <div key={i}>Filler line {i+1}</div>)}
              <button class="mt-4 px-4 py-2 bg-red-600 text-white rounded" onClick={() => setOverlay(null)}>Close</button>
            </div>
          </div></div>);
        }}>Full overlay/dialog</button> */}
        {/* {".".repeat(100).split("").map((_,i) => <div key={i}>Filler line {i+1}</div>)} */}
        <button class="fixed bottom-4 right-4 mt-4 px-3 py-3 bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white rounded-full shadow-lg"
        aria-label="Add Advisory" data-tooltip-id="tooltip" data-tooltip-content="Add Advisory" 
        onClick={() => {
          // FAB to add new advisories
          const newAdvisory = defaultAdvisory();
          invoke<string>("generate_advisory_id").then((generatedId) => {
            newAdvisory.id = generatedId;
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