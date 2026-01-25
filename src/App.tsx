import UserTable from "./components/UserTable";
import Tabs from "./components/Tabs";
import Menubar from "./components/Menubar";

export default function App() {
  return (
    <div class="select-none h-screen w-screen flex flex-col items-start justify-start bg-gray-100 dark:bg-gray-900">
      <Menubar />
      <Tabs
        tabs={[
          { id: "in-world", label: "In World", content: <UserTable /> },
          {
            id: "tickets",
            label: "Tickets",
            content: (
              <div className="text-sm text-gray-600 dark:text-gray-300">
                This is where your active mod-action tickets will appear.
              </div>
            ),
          },
          {
            id: "events",
            label: "Notices",
            content: (
              <div className="text-sm text-gray-600 dark:text-gray-300">
                Certain logged and notifiable events will appear here, such as advisories from joining users, avatar changes, messages from the world, and more.
                You may not have access to all event types.
              </div>
            ),
          },
          {
            id: "tab-2",
            label: "Tab 2",
            content: (
              <div className="text-sm text-gray-600 dark:text-gray-300">
                This is a simple unstyled placeholder for Tab 2.
              </div>
            ),
          },
          {
            id: "tab-3",
            label: "Tab 3",
            content: (
              <div className="text-sm text-gray-600 dark:text-gray-300">
                This is a simple unstyled placeholder for Tab 3.
              </div>
            ),
          },
        ]}
      />
    </div>
  );
}
