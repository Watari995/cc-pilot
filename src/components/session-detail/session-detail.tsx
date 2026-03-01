import { invoke } from "@tauri-apps/api/core";
import { useSessionStore } from "../../hooks/use-session-store";
import { formatDuration, formatTimeAgo, formatTokens } from "../../lib/formatters";
import { STATUS_COLORS, type SessionStatus } from "../../lib/types";
import styles from "./session-detail.module.css";

const STATUS_LABELS: Record<SessionStatus, string> = {
  working: "Working",
  needs_approval: "Needs approval",
  idle: "Idle",
  done: "Done",
  error: "Error",
};

const ENVIRONMENT_JUMP_LABELS: Record<string, string> = {
  terminal: "Open in Terminal",
  vscode: "Open in VS Code",
  cursor: "Open in Cursor",
  desktop: "Open in Claude Desktop",
  web: "Open in Browser",
};

export function SessionDetail() {
  const { sessions, selectedSessionId } = useSessionStore();

  if (!selectedSessionId) {
    return (
      <div className={styles.detail}>
        <div className={styles.empty}>Select a session to view details</div>
      </div>
    );
  }

  const session = sessions.get(selectedSessionId);
  if (!session) {
    return (
      <div className={styles.detail}>
        <div className={styles.empty}>Session not found</div>
      </div>
    );
  }

  const displayTitle = session.alias || session.title;

  const handleJump = () => {
    invoke("open_in_environment", { sessionId: session.id }).catch((err) => {
      console.error("Failed to open in environment:", err);
    });
  };

  return (
    <div className={styles.detail}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.projectName}>{session.projectName}</div>
        <div className={styles.titleRow}>
          <span className={styles.sessionTitle}>{displayTitle}</span>
        </div>
        {session.branchName && (
          <div className={styles.branch}>{session.branchName}</div>
        )}
        <div
          className={styles.statusBadge}
          style={{
            color: STATUS_COLORS[session.status],
            background: `${STATUS_COLORS[session.status]}20`,
          }}
        >
          <span
            className={styles.statusDot}
            style={{ backgroundColor: STATUS_COLORS[session.status] }}
          />
          {STATUS_LABELS[session.status]}
        </div>
      </div>

      {/* Approval Banner */}
      {session.status === "needs_approval" && session.approvalDetail && (
        <div className={styles.approvalBanner}>
          <div className={styles.approvalTitle}>Waiting for Approval</div>
          <div className={styles.approvalDetail}>
            Tool: {session.approvalDetail.tool}
            <br />
            {session.approvalDetail.description}
          </div>
        </div>
      )}

      {/* Metrics */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>Metrics</div>
        <div className={styles.metricsGrid}>
          {session.model && (
            <>
              <span className={styles.metricLabel}>Model</span>
              <span className={styles.metricValue}>{session.model}</span>
            </>
          )}
          <span className={styles.metricLabel}>Input Tokens</span>
          <span className={styles.metricValue}>
            {formatTokens(session.inputTokens)}
          </span>
          <span className={styles.metricLabel}>Output Tokens</span>
          <span className={styles.metricValue}>
            {formatTokens(session.outputTokens)}
          </span>
          <span className={styles.metricLabel}>Duration</span>
          <span className={styles.metricValue}>
            {formatDuration(session.startedAt, session.lastActivityAt)}
          </span>
          <span className={styles.metricLabel}>Last Activity</span>
          <span className={styles.metricValue}>
            {formatTimeAgo(session.lastActivityAt)}
          </span>
        </div>
      </div>

      {/* Active Tools */}
      {session.activeTools.length > 0 && (
        <div className={styles.section}>
          <div className={styles.sectionTitle}>Active Tools</div>
          <div className={styles.toolChips}>
            {session.activeTools.map((tool) => (
              <span key={tool} className={styles.toolChip}>
                {tool}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Jump Button */}
      <div className={styles.section}>
        <button className={styles.jumpButton} onClick={handleJump}>
          ↗ {ENVIRONMENT_JUMP_LABELS[session.environment] || "Open"}
        </button>
      </div>
    </div>
  );
}
