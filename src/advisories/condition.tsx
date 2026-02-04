import { AdvisoryCondition } from "@app/bindings/AdvisoryCondition";
import DeleteIcon from "mdi-preact/DeleteIcon";
import Dropdown from "../components/Dropdown";
import MonitorIcon from "mdi-preact/MonitorIcon";
import AndroidIcon from "mdi-preact/AndroidIcon";
import AppleIcon from "mdi-preact/AppleIcon";

export default function ConditionEditor({ condition, setCondition, removeCondition }: { 
  condition: AdvisoryCondition,
  setCondition: (condition: AdvisoryCondition) => void ,
  removeCondition?: (() => void)
}) {
  const stringDataConditionTypes = [
    "UsernameContains",
    "AvatarMayBe",
    "IsGroupMember",
    "InstanceOwner",
    "LogLinePrefix"
  ];
  return <div class="p-4 border border-gray-300 dark:border-gray-700 rounded">
    <Dropdown items={[
      //{active: condition.type === "None", set: () => setCondition({type: "None", data: null}), label: ConditionLabel({type: "None", data: null} as AdvisoryCondition)},
      {active: condition.type === "AllOf", set: () => setCondition({type: "AllOf", data: []}), label: <ConditionName condition="AllOf" />},
      {active: condition.type === "AnyOf", set: () => setCondition({type: "AnyOf", data: []}), label: <ConditionName condition="AnyOf" />},
      {active: condition.type === "Not", set: () => setCondition({type: "Not", data: {data: {type: "AnyOf", data: []}}}), label: <ConditionName condition="Not" />},
      {active: condition.type === "UsernameContains", set: () => setCondition({type: "UsernameContains", data: ""}), label: <ConditionName condition="UsernameContains" />},
      {active: condition.type === "AccountAgeAtMostDays", set: () => setCondition({type: "AccountAgeAtMostDays", data: 0}), label: <ConditionName condition="AccountAgeAtMostDays" />},
      {active: condition.type === "AvatarMayBe", set: () => setCondition({type: "AvatarMayBe", data: ""}), label: <ConditionName condition="AvatarMayBe" />},
      {active: condition.type === "IsGroupMember", set: () => setCondition({type: "IsGroupMember", data: ""}), label: <ConditionName condition="IsGroupMember" />},
      {active: condition.type === "AgeNotVerified", set: () => setCondition({type: "AgeNotVerified"}), label: <ConditionName condition="AgeNotVerified" />},
      {active: condition.type === "PlatformIs", set: () => setCondition({type: "PlatformIs", data: ""}), label: <ConditionName condition="PlatformIs" />},
      {active: condition.type === "TrustRankAtMost", set: () => setCondition({type: "TrustRankAtMost", data: "Nuisance"}), label: <ConditionName condition="TrustRankAtMost" />},
      {active: condition.type === "InstanceGroupRestricted", set: () => setCondition({type: "InstanceGroupRestricted", data: null}), label: <ConditionName condition="InstanceGroupRestricted" />},
      {active: condition.type === "InstanceOwner", set: () => setCondition({type: "InstanceOwner", data: ""}), label: <ConditionName condition="InstanceOwner" />},
      // {active: condition.type === "LogLinePrefix", set: () => setCondition({type: "LogLinePrefix", data: ""}), label: <ConditionName condition="LogLinePrefix" />}
    ]} />
    {removeCondition && <button class="float-end bg-transparent hover:bg-black/20 hover:dark:bg-white/20 text-white hover:text-red-400 transition rounded-full p-2 m-2 mt-0 me-0" onClick={() => removeCondition()}><DeleteIcon /></button>}
    {/* TODO: list of sub-conditions if this is a meta-condition */}
    {/* TODO: field(s) for condition data that appear when certain condition types are selected */}
    {(condition.type === "AllOf" || condition.type === "AnyOf") ? <div class="mt-4 border border-gray-300 dark:border-gray-700 rounded p-1">
      <div class="font-semibold mb-2">Sub-Conditions:</div>
      {(condition.data.length === 0) && <div class="italic text-gray-500">No sub-conditions defined.</div>}
      {condition.data.map((subCondition, index) => (
        <div class="mb-2" key={index}>
          <ConditionEditor condition={subCondition} setCondition={(newSubCondition) => {
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
        const newData = [...condition.data, {type: "None", data: null} as AdvisoryCondition];
        setCondition({...condition, data: newData});
      }}>Add Sub-Condition</button>
    </div>
    : condition.type === "Not" ? <div class="mt-4 border border-gray-300 dark:border-gray-700 rounded p-1">
      <div class="font-semibold mb-2">Sub-Condition:</div>
      <ConditionEditor condition={condition.data.data} setCondition={(newSubCondition) => {
        setCondition({...condition, data: {data: newSubCondition}});
      }} />
    </div>
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
    : condition.type === "TrustRankAtMost" ? <div class="mt-4">
      <label class="block font-semibold mb-1">{ConditionLabel(condition)}</label>
      <Dropdown items={[
        {active: condition.data === "Nuisance", set: () => setCondition({...condition, data: "Nuisance"}), label: <>Nuisance</>},
        {active: condition.data === "Visitor", set: () => setCondition({...condition, data: "Visitor"}), label: <>Visitor</>},
        {active: condition.data === "NewUser", set: () => setCondition({...condition, data: "NewUser"}), label: <>New User</>},
        {active: condition.data === "User", set: () => setCondition({...condition, data: "User"}), label: <>User</>},
        {active: condition.data === "KnownUser", set: () => setCondition({...condition, data: "KnownUser"}), label: <>Known User</>},
        {active: condition.data === "TrustedUser", set: () => setCondition({...condition, data: "TrustedUser"}), label: <>Trusted User</>},
        {active: condition.data === "Admin", set: () => setCondition({...condition, data: "Admin"}), label: <>Admin</>},
      ]} /> 
    </div>
    : condition.type === "PlatformIs" ? <div class="mt-4">
      <label class="block font-semibold mb-1">{ConditionLabel(condition)}</label>
      <Dropdown items={[
        {active: condition.data === "standalonewindows", set: () => setCondition({...condition, data: "standalonewindows"}), label: <><MonitorIcon class="inline-block w-4 h-4 mr-1" />PC</>},
        {active: condition.data === "android", set: () => setCondition({...condition, data: "android"}), label: <><AndroidIcon class="inline-block w-4 h-4 mr-1" />Android</>},
        {active: condition.data === "ios", set: () => setCondition({...condition, data: "ios"}), label: <><AppleIcon class="inline-block w-4 h-4 mr-1" />iOS</>},
      ]} />
    </div>
    : condition.type === "AccountAgeAtMostDays" ? <div class="mt-4">
      <label class="block font-semibold mb-1">{ConditionLabel(condition)}</label>
      <input 
        type="number"
        class="w-full p-2 border border-gray-300 dark:border-gray-700 rounded"
        value={condition.data as number}
        onInput={(e) => setCondition({...condition, data: (e.target as HTMLInputElement).valueAsNumber} as any)}
      />
    </div>
    : <pre class="font-mono">{htmlescape(condition.type)}: {htmlescape((condition as any).data)}</pre>}
  </div>;
}

function ConditionName({condition}: {condition: AdvisoryCondition | string | undefined}) {
  condition = (typeof condition === "string" ? {type: condition, data: null} as AdvisoryCondition : condition);
  if (!condition) {
    return <>Select Condition Type...</>;
  }
  if (condition.type === "AllOf") {
    return "All of...";
  } else if (condition.type === "AnyOf") {
    return "Any of...";
  } else if (condition.type === "Not") {
    return "Not...";
  } else if (condition.type === "UsernameContains") {
    return "Username contains";
  } else if (condition.type === "AccountAgeAtMostDays") {
    return "Account age at most";
  } else if (condition.type === "AvatarMayBe") {
    return <>Avatar<p class="text-sm text-gray-500">One of a list of possibly-equipped avatars</p></>;
  } else if (condition.type === "IsGroupMember") {
    return "Is member of group";
  } else if (condition.type === "AgeNotVerified") {
    return "Not 18+ age-verified";
  } else if (condition.type === "PlatformIs") {
    return "Platform";
  } else if (condition.type === "TrustRankAtMost") {
    return "Max trust rank";
  } else if (condition.type === "InstanceGroupRestricted") {
    return "Group-only or Group+ instance";
  } else if (condition.type === "InstanceOwner") {
    return "Instance owned by";
  } else if (condition.type === "LogLinePrefix") {
    return <>Log line has prefix<p class="text-sm text-gray-500">Mutually exclusive with most other conditions</p></>;
  } else {
    return condition.type;
  }
}
function ConditionLabel(condition: AdvisoryCondition) {
  if (condition.type === "UsernameContains") {
    return `Contains:`;
  } else if (condition.type === "AccountAgeAtMostDays") {
    return `Age (days):`;
  } else if (condition.type === "AvatarMayBe") {
    return `Avatar ID (avtr_***):`;
  } else if (condition.type === "IsGroupMember") {
    return `Group ID (grp_***):`;
  } else if (condition.type === "TrustRankAtMost") {
    return `Highest trust rank:`;
  } else if (condition.type === "PlatformIs") {
    return `Platform:`;
  } else if (condition.type === "InstanceOwner") {
    return `Instance owner ID (usr_*** or grp_***):`;
  } else if (condition.type === "LogLinePrefix") {
    return `Log line starts with:`;
  } else {
    return "Value:";
  }
}
function htmlescape(str: string) {
  return ((str||"null").toString()).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
} 

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