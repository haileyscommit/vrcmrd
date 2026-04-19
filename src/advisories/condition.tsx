import { AdvisoryCondition } from "@app/bindings/AdvisoryCondition";
import Dropdown from "../components/Dropdown";
import DeleteIcon from "mdi-preact/DeleteIcon";
import RightIcon from "mdi-preact/ChevronRightIcon";
import DownIcon from "mdi-preact/ChevronDownIcon";
import DragVerticalIcon from "mdi-preact/DragVerticalIcon";
import GroupConditionEditor from "./condition_group";
import AppleIcon from "mdi-preact/AppleIcon";
import AndroidIcon from "mdi-preact/AndroidIcon";
import MonitorIcon from "mdi-preact/MonitorIcon";
import { menu } from "@tauri-apps/api";
import { LogicalPosition, PhysicalPosition, Position } from "@tauri-apps/api/dpi";

export default function ConditionEditor({ condition, depth, setCondition, addSibling, removeCondition }: { 
  condition: AdvisoryCondition,
  depth: number,
  setCondition: (condition: AdvisoryCondition) => void,
  addSibling?: (condition: AdvisoryCondition) => void,
  removeCondition?: (() => void)
}) {
  const singleStringConditions: AdvisoryCondition["type"][] = ["UsernameContains", "StatusContains", "PronounContains", "AvatarMayBe", "AvatarNameContains", "IsGroupMember", "InstanceOwner"];
  function cycleConditionType(e: Event) {
    // AllOf -> AnyOf -> Not -> AllOf
    if (condition.type === "AllOf") {
      setCondition({type: "AnyOf", data: condition.data});
    } else if (condition.type === "AnyOf" && condition.data.length == 0) {
      setCondition({type: "Not", data: {data: {type: "None"}}});
    } else if (condition.type === "AnyOf" && condition.data.length <= 1) {
      setCondition({type: "Not", data: {data: condition.data[0]}});
    } else if (condition.type === "AnyOf") {
      setCondition({type: "AllOf", data: condition.data});
    } else if (condition.type === "Not") {
      setCondition({type: "AllOf", data: [condition.data.data]});
    }
    e.preventDefault();
  }
  async function showMenu(at?: PhysicalPosition | LogicalPosition | Position) {
    const cm = await menu.Menu.new({id: `ccm`, items: [
      await menu.MenuItem.new({
        text: 'Copy',
        accelerator: 'c',
        action: () => {
          window.navigator.clipboard.writeText(JSON.stringify(condition));
        },
      }),
      await menu.MenuItem.new({
        text: 'Cut',
        accelerator: 'x',
        action: () => {
          window.navigator.clipboard.writeText(JSON.stringify(condition));
          removeCondition ? removeCondition() : setCondition({type: "None"});
        },
      }),
      await menu.MenuItem.new({
        text: 'Paste here',
        accelerator: 'v',
        // Can paste into this condition if this condition is empty (to replace it) or if it's AllOf/AnyOf (to add into it)
        enabled: condition.type === "None" || condition.type === "AllOf" || condition.type === "AnyOf" || (condition.type === "Not" && condition.data.data.type === "None"),
        action: async () => {
          const text = await window.navigator.clipboard.readText();
          try {
            const parsed = JSON.parse(text);
            // TODO: validate the parsed object before accepting it
            if (condition.type === "None") {
              setCondition(parsed);
            } else if (condition.type === "AllOf" || condition.type === "AnyOf") {
              setCondition({...condition, data: [...condition.data, parsed]});
            } else if (condition.type === "Not") {
              setCondition({...condition, data: {data: parsed}});
            }
          } catch (e) {
            console.error("Failed to parse condition from clipboard", e);
          }
        },
      }),
      await menu.MenuItem.new({
        text: 'Paste after',
        accelerator: 'Shift+V',
        action: async () => {
          const text = await window.navigator.clipboard.readText();
          try {
            const parsed = JSON.parse(text);
            addSibling && addSibling(parsed);
          } catch (e) {
            console.error("Failed to parse condition from clipboard", e);
          }
        },
        enabled: !!addSibling,
      }),
      await menu.MenuItem.new({
        text: 'Duplicate',
        accelerator: 'd',
        action: () => {
          addSibling && addSibling(condition);
        },
        enabled: !!addSibling,
      }),
      await menu.Submenu.new({
        text: 'Wrap in...',
        enabled: condition.type !== "None",
        items: [
          await menu.MenuItem.new({
            text: 'All of...',
            action: () => setCondition({type: "AllOf", data: [condition]}),
          }),
          await menu.MenuItem.new({
            text: 'Any of...',
            action: () => setCondition({type: "AnyOf", data: [condition]}),
          }),
          await menu.MenuItem.new({
            text: 'Not...',
            action: () => setCondition({type: "Not", data: {data: condition}}),
          }),
        ]
      }),
      await menu.MenuItem.new({
        text: 'Unwrap',
        enabled: condition.type === "Not" || ((condition.type === "AllOf" || condition.type === "AnyOf") && condition.data.length === 1),
        action: () => {
          if (condition.type === "Not") {
            setCondition(condition.data.data);
          } else if (condition.type === "AllOf" || condition.type === "AnyOf") {
            setCondition(condition.data[0]);
          }
        },
      }),
      await menu.MenuItem.new({
        text: 'Remove',
        accelerator: 'r',
        action: () => removeCondition ? removeCondition() : setCondition({type: "None"}),
      }),
      // await menu.Submenu.new({
      //   text: 'Suppress advisory for user',
      //   enabled: false,
      //   items: [
      //     await menu.MenuItem.new({ text: 'Watchlist 1' }),
      //     await menu.MenuItem.new({ text: 'Watched avatar' }),
      //   ]
      // }),
    ]});
    cm.popup(at);
  }
  if (condition.type === "AllOf" || condition.type === "AnyOf") {
    return <details onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="border border-gray-300 dark:border-gray-700 only:rounded first:rounded-t last:rounded-b not-last:border-b-none p-2 flex flex-col gap-2 items-stretch w-full" open>
      <summary class="flex flex-row gap-2 w-full items-center">
        {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
        <RightIcon class="[details[open]>summary>&]:hidden cursor-pointer" />
        <DownIcon class="hidden [details[open]>summary>&]:inline-block" />
        {condition.type === "AllOf" 
        ? <span class="cursor-pointer inline-block font-bold text-sm border rounded-full px-2 py-1/2 text-green-400 bg-green-100 dark:bg-green-800 hover:bg-green-200 dark:hover:bg-green-700 border-green-400" onClick={cycleConditionType}>All of</span> 
        : <span class="cursor-pointer inline-block font-bold text-sm border rounded-full px-2 py-1/2 text-blue-400 bg-blue-100 dark:bg-blue-800 hover:bg-blue-200 dark:hover:bg-blue-700 border-blue-400" onClick={cycleConditionType}>Any of</span>}
        <span class="flex-grow italic text-sm text-gray-400">{condition.data.length} sub-condition{condition.data.length !== 1 && "s"}</span>
        <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
      </summary>
      <div class="mb-2 w-full">
        {condition.data.map((c, i) => <ConditionEditor key={i} depth={(depth ?? 0) + 1} condition={c} setCondition={(newCondition) => {
          const newConditions = [...condition.data];
          newConditions[i] = newCondition;
          setCondition({...condition, data: newConditions});
        }} removeCondition={() => {
          const newConditions = [...condition.data];
          newConditions.splice(i, 1);
          setCondition({...condition, data: newConditions});
        }} addSibling={(sibling) => {
          const newConditions = [...condition.data, sibling];
          setCondition({...condition, data: newConditions});
        }} />)}
      </div>
      <NewCondition setCondition={(newCondition) => setCondition({...condition, data: [...condition.data, newCondition]})} />
    </details>
  } else if (condition.type === "Not") {
    return <details onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="border border-gray-300 dark:border-gray-700 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 p-2 flex flex-col gap-2 items-stretch w-full" open>
      <summary class="flex flex-row gap-2 w-full items-center">
        {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
        <RightIcon class="[details[open]>summary>&]:hidden cursor-pointer" />
        <DownIcon class="hidden [details[open]>summary>&]:inline-block" />
        <span class="cursor-pointer font-bold text-sm border rounded-full px-2 py-1/2 text-red-400 bg-red-100 dark:bg-red-800 hover:bg-red-200 dark:hover:bg-red-700 border-red-400" onClick={cycleConditionType}>Not</span>
        <span class="flex-grow" />
        <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
      </summary>
      <div class="w-full"><ConditionEditor depth={(depth ?? 0) + 1} condition={condition.data.data} setCondition={(newCondition) => setCondition({...condition, data: {data: newCondition}})} /></div>
    </details>
  } else if (condition.type === "None") {
    return <div onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="w-full flex flex-col gap-2 items-start border border-gray-300 dark:border-gray-700 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 p-4">
      {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
      <span class="font-bold">No condition set</span>
      <NewCondition setCondition={(newCondition) => setCondition(newCondition)} />
    </div>
  } else if (singleStringConditions.includes(condition.type)) {
    return <div onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="border border-gray-300 dark:border-gray-600 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 bg-gray-200 dark:bg-gray-700 p-2 flex flex-row gap-2 items-start w-full">
      {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
      <label class="font-bold w-[32ch] overflow-hidden text-wrap my-2">{ConditionLabel(condition)}</label>
      <div class="w-full flex flex-col gap-1/2 flex-grow">
        {ConditionInputTip(condition) && <span class="text-xs italic text-gray-400">{ConditionInputTip(condition)}</span>}
        <input type="text" class="w-full bg-transparent border border-gray-300 dark:border-gray-600 rounded p-1 mt-1 flex-grow" value={(condition as any).data} onInput={(e) => setCondition({...condition, data: (e.target as HTMLInputElement).value} as any)} />
      </div>
      <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
    </div>
  } else if (condition.type === "AccountAgeAtMostDays") {
    return <div onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="border border-gray-300 dark:border-gray-600 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 bg-gray-200 dark:bg-gray-700 p-2 flex flex-row gap-2 items-stretch w-full">
      {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
      <label class="font-bold w-[32ch] overflow-hidden text-wrap my-2">{ConditionLabel(condition)}</label>
      <div class="w-full flex flex-col gap-1/2 flex-grow">
        <input type="number" class="w-full bg-transparent border border-gray-300 dark:border-gray-600 rounded p-1 mt-1 flex-grow text-end" value={(condition as any).data} onInput={(e) => setCondition({...condition, data: (e.target as HTMLInputElement).value} as any)} />
      </div>
      <span class="text-sm italic text-gray-400 self-center mx-2">days</span>
      <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
    </div>
  } else if (condition.type === "PlatformIs") {
    return <div onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="border border-gray-300 dark:border-gray-600 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 bg-gray-200 dark:bg-gray-700 p-2 flex flex-row gap-2 items-stretch w-full">
      {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
      <label class="font-bold w-[32ch] overflow-hidden text-wrap my-2">{ConditionLabel(condition)}</label>
      <div class="w-full flex flex-col gap-1/2 flex-grow">
        <Dropdown class="bg-gray-200 dark:bg-gray-800 rounded" items={[
          {active: condition.data === "standalonewindows", set: () => setCondition({...condition, data: "standalonewindows"}), label: <><MonitorIcon class="inline-block w-4 h-4 mr-2" />PC<span class="flex-grow" /></>},
          {active: condition.data === "android", set: () => setCondition({...condition, data: "android"}), label: <><AndroidIcon class="inline-block w-4 h-4 mr-2" />Android<span class="flex-grow" /></>},
          {active: condition.data === "ios", set: () => setCondition({...condition, data: "ios"}), label: <><AppleIcon class="inline-block w-4 h-4 mr-2" />iOS<span class="flex-grow" /></>},
        ]} />
      </div>
      <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
    </div>
  } else if (condition.type === "TrustRankAtMost") {
    return <div onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="border border-gray-300 dark:border-gray-600 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 bg-gray-200 dark:bg-gray-700 p-2 flex flex-row gap-2 items-stretch w-full">
      {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
      <label class="font-bold w-[32ch] overflow-hidden text-wrap my-2">{ConditionLabel(condition)}</label>
      <div class="w-full flex flex-col gap-1/2 flex-grow">
        <Dropdown class="bg-gray-200 dark:bg-gray-800 rounded" items={[
          {active: condition.data === "Nuisance", set: () => setCondition({...condition, data: "Nuisance"}), label: <>Nuisance</>},
          {active: condition.data === "Visitor", set: () => setCondition({...condition, data: "Visitor"}), label: <>Visitor</>},
          {active: condition.data === "NewUser", set: () => setCondition({...condition, data: "NewUser"}), label: <>New User</>},
          {active: condition.data === "User", set: () => setCondition({...condition, data: "User"}), label: <>User</>},
          {active: condition.data === "KnownUser", set: () => setCondition({...condition, data: "KnownUser"}), label: <>Known User</>},
          {active: condition.data === "TrustedUser", set: () => setCondition({...condition, data: "TrustedUser"}), label: <>Trusted User</>},
          {active: condition.data === "Admin", set: () => setCondition({...condition, data: "Admin"}), label: <>Admin</>},
        ]} />
      </div>
      <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
    </div>
  } else if (condition.type === "GroupCondition") {
    return <details onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="border border-gray-300 dark:border-gray-700 !border-l-green-400 border-l-4 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 flex flex-col items-stretch w-full" open>
      <summary class="flex flex-row gap-2 w-full items-center p-2">
        {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
        <RightIcon class="[details[open]>summary>&]:hidden cursor-pointer" />
        <DownIcon class="hidden [details[open]>summary>&]:inline-block" />
        <span class="font-bold w-[32ch] overflow-hidden text-wrap my-2">In a group matching this condition</span>
        <span class="flex-grow" />
        <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
      </summary>
      <div class="p-2 group-last:rounded-bl group-only:rounded-bl">
        <GroupConditionEditor condition={condition.data} setCondition={(newCondition) => setCondition({...condition, data: newCondition})} removeCondition={removeCondition} />
      </div>
    </details>
  } else if (condition.type === "AgeNotVerified" || condition.type === "InstanceGroupRestricted") {
    return <div onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="border border-gray-300 dark:border-gray-600 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 bg-gray-200 dark:bg-gray-700 p-2 flex flex-row gap-2 items-center w-full">
      {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
      <span class="font-bold w-[32ch] overflow-hidden text-wrap">{ConditionLabel(condition)}</span>
      {/* <span class="italic text-sm text-gray-400 flex-grow">No editor implemented for this condition type yet.</span> */}
      <span class="flex-grow" />
      <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
    </div>
  } else {
    return <div onContextMenu={(e) => {e.preventDefault(); e.stopPropagation(); showMenu();}} class="border border-gray-300 dark:border-gray-600 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 bg-gray-200 dark:bg-gray-700 p-2 flex flex-row gap-2 items-center w-full">
      {depth >= 2 && <ReorderConditionHandle onClick={() => showMenu()} onDrag={null} />}
      <span class="font-bold w-[32ch] overflow-hidden text-wrap">{ConditionLabel(condition)}</span>
      <span class="italic text-sm text-gray-400 flex-grow">No editor implemented for this condition type yet.</span>
      <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
    </div>
  }
}

export function ReorderConditionHandle({ onClick, onDrag }: { onClick?: () => void, onDrag: (() => void) | null | undefined }) {
  return <></>; // for now; reordering is hard
  return <button class="bg-transparent hover:bg-black/20 hover:dark:bg-white/20 text-white hover:text-gray-400 transition rounded-lg py-2 cursor-move w-6" draggable={onDrag != null}
    onDrag={onDrag || undefined}
    onClick={onClick}
    ><DragVerticalIcon size={"1.5em"} /></button>;
}

export function RemoveConditionButton({ onClick, setCondition }: { onClick: (() => void) | null | undefined, setCondition: (condition: AdvisoryCondition) => void }) {
  return <button class="bg-transparent hover:bg-black/20 hover:dark:bg-white/20 text-white hover:text-red-400 transition rounded-full p-2" onClick={() => onClick ? onClick() : setCondition({"type": "None"})}><DeleteIcon /></button>;
}

export function NewCondition({ setCondition }: { setCondition: (condition: AdvisoryCondition) => void }) {
  return <Dropdown class="block w-full" label={<>+ Add new condition...</>} items={[
      {active: false, set: () => setCondition({type: "AllOf", data: []}), label: <>All of...</>, description: <>All sub-conditions must be met. (AND between each condition.)</>},
      {active: false, set: () => setCondition({type: "AnyOf", data: []}), label: <>Any of...</>, description: <>At least one sub-condition must be met. (OR between each condition.)</>},
      {active: false, set: () => setCondition({type: "Not", data: {data: {type: "None"}}}), label: <>Not...</>, description: <>The sub-condition must NOT be met. (Inverts the sub-condition.)</>},
      {active: false, set: () => setCondition({type: "GroupCondition", data: {type: "None"}}), label: <>In a group matching condition...</>, description: <>The condition applies to each group the user is in, and if any group meets the condition, the user matches.</>},
      {active: false, set: () => setCondition({type: "UsernameContains", data: ""}), label: <>Username contains</>},
      {active: false, set: () => setCondition({type: "StatusContains", data: ""}), label: <>Status contains</>},
      {active: false, set: () => setCondition({type: "PronounContains", data: ""}), label: <>Pronouns contain</>},
      {active: false, set: () => setCondition({type: "AccountAgeAtMostDays", data: 0}), label: <>Account age</>},
      {active: false, set: () => setCondition({type: "AvatarMayBe", data: ""}), label: <>Avatar</>, description: <>One of a list of possibly-equipped avatars</>},
      {active: false, set: () => setCondition({type: "AvatarNameContains", data: ""}), label: <>Avatar name contains</>, description: <>Useful to find types of avatar that are commonly used by trolls</>},
      {active: false, set: () => setCondition({type: "IsGroupMember", data: ""}), label: <>Is member of group</>},
      {active: false, set: () => setCondition({type: "AgeNotVerified"}), label: <>Not 18+ age-verified</>, description: <>Users who have not ID-verified with VRChat</>},
      {active: false, set: () => setCondition({type: "PlatformIs", data: ""}), label: <>Platform</>},
      {active: false, set: () => setCondition({type: "TrustRankAtMost", data: "Nuisance"}), label: <>Max trust rank</>},
      {active: false, set: () => setCondition({type: "InstanceGroupRestricted", data: null}), label: <>Group-only or Group+ Instance</>},
      {active: false, set: () => setCondition({type: "InstanceOwner", data: ""}), label: <>In instance owned by</>, description: <>The owner of the instance. Either a user or a group.</>},
  ]} />
}

export const ConditionLabel = (condition: AdvisoryCondition) => {
  switch (condition.type) {
    case "UsernameContains": return <>Username contains</>;
    case "StatusContains": return <>Status contains</>;
    case "PronounContains": return <>Pronouns contain</>;
    case "AvatarMayBe": return <>Wearing avatar</>;
    case "AvatarNameContains": return <>Avatar name contains</>;
    case "IsGroupMember": return <>Is member of group</>;
    case "InstanceOwner": return <>In instance owned by</>;
    case "InstanceGroupRestricted": return <>In Group-only or Group+ instance</>;
    case "AccountAgeAtMostDays": return <>Account age</>;
    case "PlatformIs": return <>Platform</>;
    case "TrustRankAtMost": return <>Max trust rank</>;
    case "AgeNotVerified": return <>Not 18+ age-verified</>;
    default: return condition.type;
  }
}

export const ConditionInputTip = (condition: AdvisoryCondition) => {
  switch (condition.type) {
    case "AvatarMayBe": return <>Avatar ID (avtr_***)</>;
    case "IsGroupMember": return <>Group ID (grp_***)</>;
    case "InstanceOwner": return <>User or group ID (usually usr_*** or grp_***)</>;
    default: return null;
  }
}

// function htmlescape(str: string) {
//   return ((str||"null").toString()).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
// } 

export function NestedConditionTypes(condition: AdvisoryCondition): string[] {
  let types: string[] = [condition.type];
  if (condition.type === "AllOf" || condition.type === "AnyOf") {
    for (const subCondition of condition.data) {
      types = types.concat(NestedConditionTypes(subCondition));
    }
  } else if (condition.type === "Not") {
    types = types.concat(NestedConditionTypes(condition.data.data));
  }
  return types;
}