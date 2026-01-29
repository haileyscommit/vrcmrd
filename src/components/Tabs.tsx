import Preact, { ComponentChildren } from "preact";
import { useRef, useState } from "preact/hooks";

type TabItem = {
  id: string;
  label: string;
  content: ComponentChildren;
};

export default function Tabs({ tabs }: { tabs: TabItem[] }) {
  const [activeIndex, setActiveIndex] = useState(0);
  const tabRefs = useRef<Array<HTMLButtonElement | null>>([]);

  const focusAndActivate = (index: number) => {
    setActiveIndex(index);
    tabRefs.current[index]?.focus();
  };

  const onKeyDown = (e: Preact.TargetedKeyboardEvent<HTMLButtonElement>, idx: number) => {
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
    <div
      role="tablist"
      aria-label="Primary"
      className="flex flex-row space-x-1 pt-1 px-2 overflow-x-auto h-10"
    >
      {tabs.map((t, i) => (
        <button
          key={t.id}
          id={`tab-${t.id}`}
          ref={(el) => { tabRefs.current[i] = el; }}
          role="tab"
          aria-selected={activeIndex === i}
          aria-controls={`panel-${t.id}`}
          tabIndex={0}
          onClick={() => {
            setActiveIndex(i);
            // focus the corresponding tabpanel after render
            document.getElementById(`panel-${t.id}`)?.focus() ?? document.getElementById(`tab-${t.id}`)?.blur();
          }}
          onKeyDown={(e) => {
            onKeyDown(e, i);
            // when activating via Enter/Space, move focus to the panel
            if (e.key === "Enter" || e.key === " ") {
              document.getElementById(`panel-${t.id}`)?.focus() ?? document.getElementById(`tab-${t.id}`)?.blur();
            }
          }}
          className={[
            "px-4 py-2 rounded-t-md text-sm font-medium outline-none focus:not-active:ring-2 focus:ring-indigo-400 active:bg-gray-200 dark:active:bg-gray-700",
            activeIndex === i
              ? "bg-white dark:bg-gray-800 text-indigo-600 dark:text-indigo-300 border-b-2 border-indigo-500"
              : "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
          ].join(" ")}
        >
          {t.label}
        </button>
      ))}
    </div>

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
  </>;
}
