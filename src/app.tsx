import { SessionList } from "./components/session-list/session-list";
import { useSessionStore } from "./hooks/use-session-store";
import { useTauriEvents } from "./hooks/use-tauri-events";

export function App() {
  useTauriEvents();

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        width: "100vw",
      }}
    >
      {/* Header */}
      <header
        data-tauri-drag-region
        style={{
          height: "var(--header-height)",
          background: "var(--bg-secondary)",
          borderBottom: "1px solid var(--border)",
          display: "flex",
          alignItems: "center",
          paddingLeft: 80,
          paddingRight: 16,
        }}
      >
        <span
          style={{
            fontSize: 14,
            fontWeight: 600,
            color: "var(--text-primary)",
          }}
        >
          cc-pilot
        </span>
      </header>

      {/* Main */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        <SessionList />
      </div>

      {/* Status Bar */}
      <footer
        style={{
          height: "var(--statusbar-height)",
          background: "var(--bg-secondary)",
          borderTop: "1px solid var(--border)",
          display: "flex",
          alignItems: "center",
          padding: "0 var(--space-md)",
          fontSize: 11,
          color: "var(--text-secondary)",
        }}
      >
        <StatusBarContent />
      </footer>
    </div>
  );
}

function StatusBarContent() {
  const sessions = useSessionStore((s) => s.sessions);
  const total = sessions.size;
  const active = Array.from(sessions.values()).filter(
    (s) => s.status === "working" || s.status === "needs_approval",
  ).length;

  return (
    <span>
      {total} sessions &nbsp;|&nbsp; {active} active
    </span>
  );
}
