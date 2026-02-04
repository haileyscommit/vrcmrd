import { Advisory } from "@app/bindings/Advisory";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "preact/hooks";
import AdvisoryEditor from "./editor";
import { listen } from "@tauri-apps/api/event";

export default function AdvisoryList({ setOverlay, setDialog }: {
  setOverlay?: (overlay: preact.VNode|null) => void,
  setDialog?: (dialog: preact.VNode|null) => void,
}) {
  const [loading, setLoading] = useState(true);
  const [advisories, setAdvisories] = useState<Advisory[]>([]);
  const [epoch, setEpoch] = useState(0);
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
  return <div class="flex flex-col gap-4">
    {advisories.map((advisory) => (
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