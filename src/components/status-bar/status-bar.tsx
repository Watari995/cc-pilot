import { useMemo } from "react";
import { useSessionStore } from "../../hooks/use-session-store";
import { formatTimeAgo } from "../../lib/formatters";
import { STATUS_COLORS, type SessionStatus } from "../../lib/types";
import styles from "./status-bar.module.css";

const STATUS_ENTRIES: { key: SessionStatus; label: string }[] = [
  { key: "working", label: "Working" },
  { key: "needs_approval", label: "Approval" },
  { key: "idle", label: "Idle" },
  { key: "done", label: "Done" },
  { key: "error", label: "Error" },
];

export function StatusBar() {
  const sessions = useSessionStore((s) => s.sessions);

  const { counts, total, lastActivity } = useMemo(() => {
    const c: Record<string, number> = {};
    let latest = "";
    for (const s of sessions.values()) {
      c[s.status] = (c[s.status] || 0) + 1;
      if (s.lastActivityAt > latest) {
        latest = s.lastActivityAt;
      }
    }
    return { counts: c, total: sessions.size, lastActivity: latest };
  }, [sessions]);

  return (
    <div className={styles.container}>
      <div className={styles.statusGroup}>
        {STATUS_ENTRIES.map(({ key, label }) => {
          const count = counts[key] || 0;
          if (count === 0) return null;
          return (
            <span key={key} className={styles.statusItem}>
              <span
                className={styles.statusDot}
                style={{ backgroundColor: STATUS_COLORS[key] }}
              />
              <span className={styles.statusLabel}>
                {count} {label}
              </span>
            </span>
          );
        })}
      </div>

      <div className={styles.meta}>
        <span>{total} sessions</span>
        {lastActivity && (
          <>
            <span>|</span>
            <span>Last: {formatTimeAgo(lastActivity)}</span>
          </>
        )}
      </div>
    </div>
  );
}
