import { useRef, useState } from "preact/hooks";
import { TabItem } from "./Tabs";

export default function SideTabs({ tabs, loading, title }: { tabs: TabItem[]; loading: boolean; title?: string }) {
  const [activeIndex, setActiveIndex] = useState(0);
  const tabRefs = useRef<Array<HTMLButtonElement | null>>([]);

  const focusAndActivate = (index: number) => {
    setActiveIndex(index);
    tabRefs.current[index]?.focus();
  };

  const onKeyDown = (e: preact.TargetedKeyboardEvent<HTMLButtonElement>, idx: number) => {
    const last = tabs.length - 1;
    if (e.key === "ArrowRight") {
      e.preventDefault();
      focusAndActivate(idx === last ? 0 : idx + 1);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      focusAndActivate(idx === 0 ? last : idx - 1);
    } else if (e.key === "Home") {
      e.preventDefault();
      focusAndActivate(0);
    } else if (e.key === "End") {
      e.preventDefault();
      focusAndActivate(last);
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      setActiveIndex(idx);
    }
  };

  return <>
    <aside className="w-56 bg-white dark:bg-gray-800">
      {title && <div className="p-4">
        <h2 className="text-lg font-medium text-gray-900 dark:text-gray-100">{title}</h2>
      </div>}

      <nav className="p-2 space-y-1" role="tablist" aria-label="Tabs">
        {tabs.map((t, i) => (
          <button
            key={t.id}
            role="tab"
            id={`tab-${t.id}`}
            ref={(el) => { tabRefs.current[i] = el; }}
            aria-selected={activeIndex === i}
            aria-controls={`panel-${t.id}`}
            onClick={() => {
              focusAndActivate(i);
              document.getElementById(`panel-${t.id}`)?.focus() ?? document.getElementById(`tab-${t.id}`)?.blur();
            }}
            onKeyDown={(e) => {
              onKeyDown(e, i);
              if (e.key === "Enter" || e.key === " ") {
                document.getElementById(`panel-${t.id}`)?.focus() ?? document.getElementById(`tab-${t.id}`)?.blur();
              }
            }}
            disabled={loading}
            className={
            `w-full text-left px-3 py-2 rounded-md flex items-center gap-2 text-sm outline-none not-active:focus:ring-2 not-active:focus:ring-indigo-400 hover:bg-gray-50 dark:hover:bg-gray-700 ` +
            (loading ? "opacity-50 pointer-events-none cursor-not-allowed " : "") +
            (activeIndex === i
              ? "bg-gray-100 dark:bg-gray-700/50 text-indigo-600 dark:text-indigo-300 font-semibold"
              : "text-gray-600 dark:text-gray-400")
            }
          >
            {t.label}
          </button>
        ))}
      </nav>
    </aside>

    <main className="flex-1 min-h-0 flex flex-col">
      {/* There was a page title here, but we don't need it. So it's gone now. */}
      <section className="flex-1 min-h-0 overflow-auto p-6 bg-gray-100 dark:bg-gray-900">
        {tabs.map((t, i) => (
          <div
            key={t.id}
            role="tabpanel"
            id={`panel-${t.id}`}
            aria-labelledby={`tab-${t.id}`}
            hidden={activeIndex !== i}
            class="contents"
          >
            {t.content}
          </div>
        ))}
        </section>
    </main>
  </>;
}