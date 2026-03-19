import { Advisory } from "@app/bindings/Advisory";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "preact/hooks";
import AdvisoryEditor from "./editor";
import { listen } from "@tauri-apps/api/event";
import { fuzzy, search } from "fast-fuzzy";
import { NestedConditionTypes } from "./condition";

export default function AdvisoryList({ setOverlay, setDialog }: {
  setOverlay?: (overlay: preact.VNode|null) => void,
  setDialog?: (dialog: preact.VNode|null) => void,
}) {
  const [loading, setLoading] = useState(true);
  const [advisories, setAdvisories] = useState<Advisory[]>([]);
  const [epoch, setEpoch] = useState(0);
  const [filter, setFilter] = useState("");
  useEffect(() => {
    invoke<Advisory[]>("get_advisories").then((advisories) => {
      setAdvisories(advisories);
      setLoading(false);
    });
    const updateListener = listen("vrcmrd:advisories_updated", (_) => {
      setEpoch((e) => e + 1);
    });
    return () => {
      updateListener.then((unlisten) => unlisten());
    }
  }, [epoch]);
  if (loading) {
    return <div>Loading advisories...</div>;
  }

  const filtered = useMemo(() => {
    if (!filter) return advisories;
    function autoTags(advisory: Advisory): string[] {
      const tags = [...advisory.tags];
      if (advisory.active) tags.push("active");
      if (advisory.private) tags.push("private");
      const conditions = NestedConditionTypes(advisory.condition);
      tags.push(...conditions);
      return tags;
    }
    // TODO: support negative search (e.g. "active -private" to find active advisories that aren't private)
    return search(filter, advisories, {
      ignoreCase: true, 
      keySelector: (advisory) => `${advisory.name} ${advisory.message_template} ${autoTags(advisory).join(" ")}`,
    });
  }, [filter, advisories]);

  return <div class="flex flex-col gap-4">
    <input id="filter-input" type="text" placeholder="Filter advisories by name or tag..." class="p-2 border border-gray-300 dark:border-gray-700 rounded" onInput={(e) => {
      const filter = (e.target as HTMLInputElement).value.toLowerCase();
      filter ? setFilter(filter) : setFilter("");
    }} />
    {filtered.map((advisory) => (
      <div key={advisory.id} class="p-4 border border-gray-300 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800 transition rounded"
      onClick={(_) => {
        setOverlay?.(<AdvisoryEditor advisory={advisory} setOverlay={setOverlay} setDialog={setDialog} />);
      }}>
        <h2 class="text-xl font-bold mb-2">{advisory.name}</h2>
        <p class="mb-2 text-sm text-gray-600 dark:text-gray-400">Message: {advisory.message_template}</p>
      </div>
    ))}
  </div>;
}