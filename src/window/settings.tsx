import { invoke } from "@tauri-apps/api/core";
import "../index.css";
import { render } from "preact";
import { useState } from "preact/hooks";
import { useOverlayScrollbars } from "../components/OverlayScrollbarsHook";
import SideTabs from "../components/SideTabs";
import NetworkingSettingsPage from "./settings/networking";

function Settings() {
	useOverlayScrollbars();
	const [loading, setLoading] = useState(false);

	return (
		<div className="h-screen select-none flex bg-gray-100 text-gray-600 dark:bg-gray-900 dark:text-gray-300">
			<SideTabs tabs={[
				{ id: "networking", label: "Networking", content: <NetworkingSettingsPage loading={loading} setLoading={setLoading} /> },
				// The two below are basic placeholders for testing
				{ id: "tab2", label: "Tab 2", content: <div className="h-full w-full flex items-center justify-center">
							<p className="text-center text-gray-600 dark:text-gray-300">Centered content for Tab 2</p>
						</div> },
				{ id: "tab3", label: "Tab 3", content: <div className="space-y-4">
							<p className="text-sm text-gray-600 dark:text-gray-300">Scrollable content below</p>
							<div className="space-y-2">
								{Array.from({ length: 50 }).map((_, i) => (
									<div key={i} className="p-3 bg-white dark:bg-gray-800 rounded shadow-sm border border-gray-200 dark:border-gray-700 text-sm text-gray-900 dark:text-gray-100">
										Item {i + 1} — Lorem ipsum dolor sit amet, consectetur adipiscing elit.
									</div>
								))}
							</div>
						</div> },
			]} loading={loading} title="Settings" />	
		</div>
	);
}

render(<Settings />, document.getElementById("root")!);