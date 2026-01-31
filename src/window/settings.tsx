import { invoke } from "@tauri-apps/api/core";
import "../index.css";
import { render } from "preact";
import { useState } from "preact/hooks";
import { useOverlayScrollbars } from "../components/OverlayScrollbarsHook";

function Settings() {
	useOverlayScrollbars();
	const tabs = ["Networking", "Tab 2", "Tab 3"];
	const [active, setActive] = useState(0);
	const [username, setUsername] = useState("");
	const [password, setPassword] = useState("");
	const [loading, setLoading] = useState(false);

	return (
		<div className="h-screen select-none flex bg-gray-100 text-gray-600 dark:bg-gray-900 dark:text-gray-300">
			<aside className="w-56 bg-white dark:bg-gray-800">
				<div className="p-4">
					<h2 className="text-lg font-medium text-gray-900 dark:text-gray-100">Settings</h2>
				</div>

				<nav className="p-2 space-y-1" role="tablist" aria-label="Settings tabs">
					{tabs.map((label, i) => (
						<button
							key={label}
							role="tab"
							aria-selected={active === i}
							aria-controls={`panel-${i}`}
							onClick={() => setActive(i)}
							disabled={loading}
							className={
							`w-full text-left px-3 py-2 rounded-md flex items-center gap-2 text-sm outline-none not-active:focus:ring-2 not-active:focus:ring-indigo-400 hover:bg-gray-50 dark:hover:bg-gray-700 ` +
							(loading ? "opacity-50 pointer-events-none cursor-not-allowed " : "") +
							(active === i
								? "bg-gray-100 dark:bg-gray-700/50 text-indigo-600 dark:text-indigo-300 font-semibold"
								: "text-gray-600 dark:text-gray-400")
							}
						>
							{label}
						</button>
					))}
				</nav>
			</aside>

			<main className="flex-1 min-h-0 flex flex-col">
				{/* <div className="p-4 bg-white dark:bg-gray-800">
					<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">{tabs[active]}</h3>
				</div> */}

				{/* Content area: each panel can scroll independently */}
				<section className="flex-1 min-h-0 overflow-auto p-6 bg-gray-100 dark:bg-gray-900">
					{/* Tab 1: default/top-left positioned content */}
					<div
						id="panel-0"
						role="tabpanel"
						hidden={active !== 0}
						className={active === 0 ? "" : "hidden"}
					>
						<div className={`space-y-4 ${loading ? "opacity-50 pointer-events-none cursor-wait" : ""}`}>
							<h2 className="text-lg font-medium text-gray-900 dark:text-gray-100">VRChat Credentials</h2>
							<p className="text-sm text-gray-600 dark:text-gray-300">We need these to access the VRChat API, which is used to get information about other users in the instance.</p>
							<div className="space-y-2">
								<label className="block text-xs text-gray-600 dark:text-gray-400">Username</label>
								<input disabled={loading} className="w-full rounded-md bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 px-3 py-2 border border-gray-300 dark:border-gray-700" placeholder="VRChat Username" value={username} onInput={(e) => setUsername(e.currentTarget.value)} />
							</div>
							<div className="space-y-2">
								<label className="block text-xs text-gray-600 dark:text-gray-400">Password</label>
								<input disabled={loading} type="password" className="w-full rounded-md bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 px-3 py-2 border border-gray-300 dark:border-gray-700" placeholder="VRChat Password" value={password} onInput={(e) => setPassword(e.currentTarget.value)} />
							</div>
							<div className="flex flex-row gap-2">
								<button className="mt-4 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-md text-sm font-medium" onClick={async () => {
									setLoading(true);
									try {
										await invoke("update_credentials", { username, password });
										setUsername("");
										setPassword("");
										alert("You are now logged in!");
									} catch (e) {
										console.error("Failed to update credentials:", e);
										alert("Could not log in. Please check your username and password, and try again.");
									} finally {
										setLoading(false);
									}
								}} disabled={loading}>{loading ? "Saving..." : "Save & Login"}</button>
								<span className="flex-1"></span>	
								<button className="mt-4 px-4 py-2 text-red-600 hover:bg-red-500/20 rounded-md text-sm font-medium" onClick={async () => {
									setUsername("");
									setPassword("");
									setLoading(true);
									try {
										await invoke("logout", {});
										alert("You have been logged out.");
									} catch (e) {
										console.error("Failed to log out:", e);
										alert("Could not log out. Please try again.");
									} finally {
										setLoading(false);
									}
								}} disabled={loading}>Log Out</button>
								</div>
						</div>
					</div>

					{/* Tab 2: vertically & horizontally centered */}
					<div
						id="panel-1"
						role="tabpanel"
						hidden={active !== 1}
						className={`${active === 1 ? "flex-1 flex" : "hidden"}`}
					>
						<div className="h-full w-full flex items-center justify-center">
							<p className="text-center text-gray-600 dark:text-gray-300">Centered content for Tab 2</p>
						</div>
					</div>

					{/* Tab 3: content with lots of items to test scrolling */}
					<div
						id="panel-2"
						role="tabpanel"
						hidden={active !== 2}
						className={active === 2 ? "" : "hidden"}
					>
						<div className="space-y-4">
							<p className="text-sm text-gray-600 dark:text-gray-300">Scrollable content below</p>
							<div className="space-y-2">
								{Array.from({ length: 50 }).map((_, i) => (
									<div key={i} className="p-3 bg-white dark:bg-gray-800 rounded shadow-sm border border-gray-200 dark:border-gray-700 text-sm text-gray-900 dark:text-gray-100">
										Item {i + 1} — Lorem ipsum dolor sit amet, consectetur adipiscing elit.
									</div>
								))}
							</div>
						</div>
					</div>
				</section>
			</main>
		</div>
	);
}

render(<Settings />, document.getElementById("root")!);