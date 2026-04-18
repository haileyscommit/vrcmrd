import Dropdown from "../components/Dropdown";
import DeleteIcon from "mdi-preact/DeleteIcon";
import RightIcon from "mdi-preact/ChevronRightIcon";
import DownIcon from "mdi-preact/ChevronDownIcon";
import { AdvisoryGroupCondition } from "@app/bindings/AdvisoryGroupCondition";
import { AdvisoryCondition } from "@app/bindings/AdvisoryCondition";

export default function GroupConditionEditor({ condition, setCondition, removeCondition }: { 
  condition: AdvisoryGroupCondition,
  setCondition: (condition: AdvisoryGroupCondition) => void,
  removeCondition?: (() => void)
}) {
  const singleStringConditions: AdvisoryGroupCondition["type"][] = ["NameContains", "Id", "OwnerIs"];
  function cycleConditionType(e: Event) {
    // AllOf -> AnyOf -> Not -> AllOf
    if (condition.type === "AllOf") {
      setCondition({type: "AnyOf", data: condition.data});
    } else if (condition.type === "AnyOf" && condition.data.length < 2) {
      setCondition({type: "Not", data: {data: condition.data[0]}});
    } else if (condition.type === "AnyOf") {
      setCondition({type: "AllOf", data: condition.data});
    } else if (condition.type === "Not") {
      setCondition({type: "AllOf", data: [condition.data.data]});
    }
    e.preventDefault();
  }
  if (condition.type === "AllOf" || condition.type === "AnyOf") {
    return <details class="border border-gray-300 dark:border-gray-700 only:rounded first:rounded-t last:rounded-b not-last:border-b-none p-2 flex flex-col gap-2 items-stretch w-full" open>
      <summary class="flex flex-row gap-2 w-full items-center">
        <RightIcon class="[details[open]>summary>&]:hidden cursor-pointer" />
        <DownIcon class="hidden [details[open]>summary>&]:inline-block" />
        {condition.type === "AllOf" 
        ? <span class="cursor-pointer inline-block font-bold text-sm border rounded-full px-2 py-1/2 text-green-400 bg-green-100 dark:bg-green-800 hover:bg-green-200 dark:hover:bg-green-700 border-green-400" onClick={cycleConditionType}>All of</span> 
        : <span class="cursor-pointer inline-block font-bold text-sm border rounded-full px-2 py-1/2 text-blue-400 bg-blue-100 dark:bg-blue-800 hover:bg-blue-200 dark:hover:bg-blue-700 border-blue-400" onClick={cycleConditionType}>Any of</span>}
        <span class="flex-grow italic text-sm text-gray-400">{condition.data.length} sub-condition{condition.data.length !== 1 && "s"}</span>
        <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
      </summary>
      <div class="mb-2 w-full">
        {condition.data.map((c, i) => <GroupConditionEditor key={i} condition={c} setCondition={(newCondition) => {
          const newConditions = [...condition.data];
          newConditions[i] = newCondition;
          setCondition({...condition, data: newConditions});
        }} removeCondition={() => {
          const newConditions = [...condition.data];
          newConditions.splice(i, 1);
          setCondition({...condition, data: newConditions});
        }} />)}
      </div>
      <NewCondition setCondition={(newCondition) => setCondition({...condition, data: [...condition.data, newCondition]})} />
    </details>
  } else if (condition.type === "Not") {
    return <details class="border border-gray-300 dark:border-gray-700 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 p-2 flex flex-col gap-2 items-stretch w-full" open>
      <summary class="flex flex-row gap-2 w-full items-center">
        <RightIcon class="[details[open]>summary>&]:hidden cursor-pointer" />
        <DownIcon class="hidden [details[open]>summary>&]:inline-block" />
        <span class="cursor-pointer font-bold text-sm border rounded-full px-2 py-1/2 text-red-400 bg-red-100 dark:bg-red-800 hover:bg-red-200 dark:hover:bg-red-700 border-red-400" onClick={cycleConditionType}>Not</span>
        <span class="flex-grow" />
        <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
      </summary>
      <div class="w-full"><GroupConditionEditor condition={condition.data.data} setCondition={(newCondition) => setCondition({...condition, data: {data: newCondition}})} /></div>
    </details>
  } else if (condition.type === "None") {
    return <div class="w-full flex flex-col gap-2 items-start border border-gray-300 dark:border-gray-700 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 p-4">
      <span class="font-bold">No condition set</span>
      <NewCondition setCondition={(newCondition) => setCondition(newCondition)} />
    </div>
  } else if (singleStringConditions.includes(condition.type)) {
    return <div class="border border-gray-300 dark:border-gray-600 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 bg-gray-200 dark:bg-gray-700 p-2 flex flex-row gap-2 items-start w-full">
      <label class="font-bold w-[32ch] overflow-hidden text-wrap my-2">{ConditionLabel(condition)}</label>
      <div class="w-full flex flex-col gap-1/2 flex-grow">
        {ConditionInputTip(condition) && <span class="text-xs italic text-gray-400">{ConditionInputTip(condition)}</span>}
        <input type="text" class="w-full bg-transparent border border-gray-300 dark:border-gray-600 rounded p-1 mt-1 flex-grow" value={(condition as any).data} onInput={(e) => setCondition({...condition, data: (e.target as HTMLInputElement).value} as any)} />
      </div>
      <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
    </div>
  } else {
    return <div class="border border-gray-300 dark:border-gray-600 only:rounded first:rounded-t last:rounded-b not-last:border-b-0 bg-gray-200 dark:bg-gray-700 p-2 flex flex-row gap-2 items-center w-full">
      <span class="font-bold w-[32ch] overflow-hidden text-wrap">{ConditionLabel(condition)}</span>
      <span class="italic text-sm text-gray-400 flex-grow">No editor implemented for this condition type yet.</span>
      <RemoveConditionButton onClick={removeCondition} setCondition={setCondition} />
    </div>
  }
}

export function RemoveConditionButton({ onClick, setCondition }: { onClick: (() => void) | null | undefined, setCondition: (condition: AdvisoryGroupCondition) => void }) {
  return <button class="bg-transparent hover:bg-black/20 hover:dark:bg-white/20 text-white hover:text-red-400 transition rounded-full p-2" onClick={() => onClick ? onClick() : setCondition({"type": "None"})}><DeleteIcon /></button>;
}

export function NewCondition({ setCondition }: { setCondition: (condition: AdvisoryGroupCondition) => void }) {
  return <Dropdown class="block w-full" label={<>+ Add new group condition...</>} items={[
      {active: false, set: () => setCondition({type: "AllOf", data: []}), label: <>All of...</>, description: <>All sub-conditions must be met. (AND between each condition.)</>},
      {active: false, set: () => setCondition({type: "AnyOf", data: []}), label: <>Any of...</>, description: <>At least one sub-condition must be met. (OR between each condition.)</>},
      {active: false, set: () => setCondition({type: "Not", data: {data: {type: "None"}}}), label: <>Not...</>, description: <>The sub-condition must NOT be met. (Inverts the sub-condition.)</>},
      {active: false, set: () => setCondition({type: "NameContains", data: ""}), label: <>Group name contains</>},
      {active: false, set: () => setCondition({type: "Id", data: ""}), label: <>Group ID is</>, description: <>The group has the given ID (e.g. "grp_***"). You should use "Is member of group" instead if you're checking for membership in a known group (it's far simpler). This is instead for cases where you want to match some conditions but exclude certain known groups.</>},
      {active: false, set: () => setCondition({type: "OwnerIs", data: ""}), label: <>Group owner is</>, description: <>The owner of the group. Useful for conditioning advisories for groups on its owner, such as known abusers or trusted community members.</>},
  ]} />
}

export const ConditionLabel = (condition: AdvisoryGroupCondition) => {
  switch (condition.type) {
    case "Id": return <>Group ID is</>;
    case "OwnerIs": return <>Group owner is</>;
    case "NameContains": return <>Group name contains</>;
    default: return condition.type;
  }
}

export const ConditionInputTip = (condition: AdvisoryGroupCondition) => {
  switch (condition.type) {
    case "Id": return <>Group ID (grp_***)</>;
    case "OwnerIs": return <>User ID (usually usr_***)</>;
    default: return null;
  }
}

// function htmlescape(str: string) {
//   return ((str||"null").toString()).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
// } 

export function NestedGroupConditionTypes(condition: AdvisoryGroupCondition): string[] {
  let types: string[] = [condition.type];
  if (condition.type === "AllOf" || condition.type === "AnyOf") {
    for (const subCondition of condition.data) {
      types = types.concat(NestedGroupConditionTypes(subCondition));
    }
  } else if (condition.type === "Not") {
    types = types.concat(NestedGroupConditionTypes(condition.data.data));
  }
  return types;
}

export function NestedGroupConditionTypesAlt(condition: AdvisoryCondition): string[] {
  let types: string[] = [];
  if (condition.type === "AllOf" || condition.type === "AnyOf") {
    for (const subCondition of condition.data) {
      types = types.concat(NestedGroupConditionTypesAlt(subCondition));
    }
  } else if (condition.type === "Not") {
    types = types.concat(NestedGroupConditionTypesAlt(condition.data.data));
  }
  if (condition.type === "GroupCondition") {
    let nestedTypes = NestedGroupConditionTypes(condition.data);
    types = types.concat(nestedTypes);
  }
  return types;
}