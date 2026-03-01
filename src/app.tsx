import { useEffect, useState } from "react";
import { SessionList } from "./components/session-list/session-list";
import { Settings } from "./components/settings/settings";
import { useSessionStore } from "./hooks/use-session-store";
import { useSettingsStore } from "./hooks/use-settings";
import { useTauriEvents } from "./hooks/use-tauri-events";

export function App() {
  useTauriEvents();

  const [showSettings, setShowSettings] = useState(false);
  const loadSettings = useSettingsStore((s) => s.load);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

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
        <span style={{ flex: 1 }} />
        <button
          onClick={() => setShowSettings((v) => !v)}
          style={{
            width: 32,
            height: 32,
            border: "none",
            background: showSettings ? "var(--bg-elevated)" : "transparent",
            color: showSettings ? "var(--text-primary)" : "var(--text-secondary)",
            fontSize: 16,
            cursor: "pointer",
            borderRadius: "var(--radius-badge)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
          title="Settings"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="8" cy="8" r="2.5" />
            <path d="M6.8 1.5h2.4l.4 1.7a5.5 5.5 0 0 1 1.4.8l1.6-.6 1.2 2.1-1.2 1.1a5.5 5.5 0 0 1 0 1.6l1.2 1.1-1.2 2.1-1.6-.6a5.5 5.5 0 0 1-1.4.8l-.4 1.7H6.8l-.4-1.7a5.5 5.5 0 0 1-1.4-.8l-1.6.6-1.2-2.1 1.2-1.1a5.5 5.5 0 0 1 0-1.6L2.2 5.5l1.2-2.1 1.6.6a5.5 5.5 0 0 1 1.4-.8l.4-1.7Z" />
          </svg>
        </button>
      </header>

      {/* Main */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        {showSettings ? (
          <Settings onClose={() => setShowSettings(false)} />
        ) : (
          <SessionList />
        )}
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
