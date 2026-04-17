import { AdvisoryGroupCondition } from "@app/bindings/AdvisoryGroupCondition";
import DeleteIcon from "mdi-preact/DeleteIcon";
import Dropdown from "../components/Dropdown";
import { AdvisoryCondition } from "@app/bindings/AdvisoryCondition";

export default function GroupConditionEditor({ condition, setCondition, removeCondition }: { 
  condition: AdvisoryGroupCondition,
  setCondition: (condition: AdvisoryGroupCondition) => void ,
  removeCondition?: (() => void)
}) {
  const stringDataConditionTypes = [
    "NameContains",
    "Id",
    "OwnerIs",
  ];
  return <div class="p-4 border border-gray-300 dark:border-gray-700 rounded">
    <Dropdown items={[
      //{active: condition.type === "None", set: () => setCondition({type: "None", data: null}), label: ConditionLabel({type: "None", data: null} as AdvisoryCondition)},
      {active: condition.type === "AllOf", set: () => setCondition({type: "AllOf", data: []}), label: <>All of...</>, description: <>All sub-conditions must be met. (AND between each condition.)</>},
      {active: condition.type === "AnyOf", set: () => setCondition({type: "AnyOf", data: []}), label: <>Any of...</>, description: <>At least one sub-condition must be met. (OR between each condition.)</>},
      {active: condition.type === "Not", set: () => setCondition({type: "Not", data: {data: {type: "AnyOf", data: []}}}), label: <>Not...</>, description: <>The sub-condition must NOT be met. (Inverts the sub-condition.)</>},
      {active: condition.type === "NameContains", set: () => setCondition({type: "NameContains", data: ""}), label: <>Group name contains</>},
      {active: condition.type === "Id", set: () => setCondition({type: "Id", data: ""}), label: <>Group ID is</>, description: <>The group has the given ID (e.g. "grp_***"). You should use "Is member of group" instead if you're checking for membership in a known group (it's far simpler). This is instead for cases where you want to match some conditions but exclude certain known groups.</>},
      {active: condition.type === "OwnerIs", set: () => setCondition({type: "OwnerIs", data: ""}), label: <>Group owner is</>, description: <>The owner of the group. Useful for conditioning advisories for groups on its owner, such as known abusers or trusted community members.</>},
    ]} />
    {removeCondition && <button class="float-end bg-transparent hover:bg-black/20 hover:dark:bg-white/20 text-white hover:text-red-400 transition rounded-full p-2 m-2 me-0" onClick={() => removeCondition()}><DeleteIcon /></button>}
    {/* TODO: list of sub-conditions if this is a meta-condition */}
    {/* TODO: field(s) for condition data that appear when certain condition types are selected */}
    {(condition.type === "AllOf" || condition.type === "AnyOf") ? <div class="mt-4 border border-gray-300 dark:border-gray-700 rounded p-1"><details open>
      <summary class="font-semibold mb-2">Sub-Conditions:</summary>
      {(condition.data.length === 0) && <div class="italic text-gray-500">No sub-conditions defined.</div>}
      {condition.data.map((subCondition, index) => (
        <div class="mb-2" key={index}>
          <GroupConditionEditor condition={subCondition} setCondition={(newSubCondition) => {
            const newData = [...condition.data];
            newData[index] = newSubCondition;
            setCondition({...condition, data: newData});
          }} removeCondition={() => {
            const newData = [...condition.data];
            newData.splice(index, 1);
            setCondition({...condition, data: newData});
          }} />
        </div>
      ))}
      <button class="mt-2 px-3 py-1 bg-blue-600 text-white rounded" onClick={() => {
        const newData = [...condition.data, {type: "None", data: null} as AdvisoryGroupCondition];
        setCondition({...condition, data: newData});
      }}>Add Sub-Condition</button>
      </details>
    </div>
    : condition.type === "Not" ? <>
      <div class="font-semibold mt-4 mb-2">Sub-Condition:</div>
      <GroupConditionEditor condition={condition.data.data} setCondition={(newSubCondition) => {
        setCondition({...condition, data: {data: newSubCondition}});
      }} />
    </>
    // Single-string data conditions
    : stringDataConditionTypes.includes(condition.type) ? <div class="mt-4">
      <label class="block font-semibold mb-1">{ConditionLabel(condition)}</label>
      <input 
        type="text"
        class="w-full p-2 border border-gray-300 dark:border-gray-700 rounded"
        value={(condition as any).data as string}
        onInput={(e) => setCondition({...condition, data: (e.target as HTMLInputElement).value} as any)}
      />
    </div>
    : <pre class="font-mono">{htmlescape(condition.type)}: {htmlescape((condition as any).data)}</pre>}
  </div>;
}

function ConditionLabel(condition: AdvisoryGroupCondition) {
  if (condition.type === "NameContains") {
    return `Group name contains:`;
  } else if (condition.type === "Id") {
    return `Group ID (grp_***):`;
  } else if (condition.type === "OwnerIs") {
    return `Group owner (usually usr_***):`;
  } else {
    return "Value:";
  }
}
function htmlescape(str: string) {
  return ((str||"null").toString()).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
} 

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