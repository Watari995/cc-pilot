import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSessionStore } from "../../hooks/use-session-store";
import {
  formatDuration,
  formatTimeAgo,
  formatTokens,
} from "../../lib/formatters";
import {
  STATUS_COLORS,
  type Environment,
  type Session,
  type SessionStatus,
} from "../../lib/types";
import { EnvironmentIcon } from "../common/environment-icon";
import styles from "./session-list.module.css";

const ENVIRONMENT_FILTERS: Array<Environment> = [
  "terminal",
  "vscode",
  "cursor",
  "desktop",
  "web",
];

const STATUS_LABELS: Record<SessionStatus, string> = {
  working: "Working",
  needs_approval: "Approval",
  idle: "Idle",
  done: "Done",
  error: "Error",
};

export function SessionList() {
  const { sessions, environmentFilter, setEnvironmentFilter } =
    useSessionStore();
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const sessionList = Array.from(sessions.values())
    .filter(
      (s) => environmentFilter === "all" || s.environment === environmentFilter,
    )
    .sort((a, b) => b.lastActivityAt.localeCompare(a.lastActivityAt));

  const toggleExpand = (id: string) => {
    setExpandedId((prev) => (prev === id ? null : id));
  };

  return (
    <div className={styles.container}>
      <div className={styles.filterBar}>
        <button
          className={`${styles.filterButton} ${
            environmentFilter === "all" ? styles.filterButtonActive : ""
          }`}
          onClick={() => setEnvironmentFilter("all")}
        >
          All
        </button>
        {ENVIRONMENT_FILTERS.map((env) => (
          <button
            key={env}
            className={`${styles.filterButton} ${
              environmentFilter === env ? styles.filterButtonActive : ""
            }`}
            onClick={() => setEnvironmentFilter(env)}
          >
            <EnvironmentIcon environment={env} size={20} />
          </button>
        ))}
      </div>

      <div className={styles.list}>
        {sessionList.length === 0 ? (
          <div className={styles.empty}>No sessions found</div>
        ) : (
          sessionList.map((session) => (
            <SessionCard
              key={session.id}
              session={session}
              isExpanded={session.id === expandedId}
              onToggle={() => toggleExpand(session.id)}
            />
          ))
        )}
      </div>
    </div>
  );
}

function SessionCard({
  session,
  isExpanded,
  onToggle,
}: {
  session: Session;
  isExpanded: boolean;
  onToggle: () => void;
}) {
  const displayTitle = session.alias || session.title;
  const statusColor = STATUS_COLORS[session.status];

  const handleJump = () => {
    invoke("open_in_environment", { sessionId: session.id }).catch((err) => {
      console.error("Failed to open in environment:", err);
    });
  };

  return (
    <div className={styles.card}>
      {/* Collapsed row - always visible */}
      <div className={styles.cardRow} onClick={handleJump}>
        <EnvironmentIcon environment={session.environment} size={32} />

        <div className={styles.cardMain}>
          <div className={styles.cardTitleRow}>
            <span className={styles.projectName}>{session.projectName}</span>
            {session.branchName && (
              <span className={styles.branchName}>{session.branchName}</span>
            )}
          </div>
          <div className={styles.title}>{displayTitle}</div>
        </div>

        <div className={styles.cardMeta}>
          <span
            className={styles.statusLabel}
            style={{ color: statusColor }}
          >
            <span
              className={styles.statusDot}
              style={{ backgroundColor: statusColor }}
            />
            {STATUS_LABELS[session.status]}
          </span>
          <span className={styles.metaItem}>
            {formatDuration(session.startedAt, session.lastActivityAt)}
          </span>
        </div>

        <button
          className={`${styles.expandBtn} ${isExpanded ? styles.expandBtnOpen : ""}`}
          onClick={(e) => {
            e.stopPropagation();
            onToggle();
          }}
        >
          ▼
        </button>
      </div>

      {/* Expanded detail */}
      {isExpanded && (
        <div className={styles.detail}>
          {/* Metrics */}
          <div className={styles.metricsRow}>
            {session.model && (
              <div className={styles.metric}>
                <span className={styles.metricLabel}>MODEL</span>
                <span className={styles.metricValue}>{session.model}</span>
              </div>
            )}
            <div className={styles.metric}>
              <span className={styles.metricLabel}>INPUT</span>
              <span className={styles.metricValue}>
                {formatTokens(session.inputTokens)} tokens
              </span>
            </div>
            <div className={styles.metric}>
              <span className={styles.metricLabel}>OUTPUT</span>
              <span className={styles.metricValue}>
                {formatTokens(session.outputTokens)} tokens
              </span>
            </div>
            <div className={styles.metric}>
              <span className={styles.metricLabel}>DURATION</span>
              <span className={styles.metricValue}>
                {formatDuration(session.startedAt, session.lastActivityAt)}
              </span>
            </div>
            <div className={styles.metric}>
              <span className={styles.metricLabel}>LAST ACTIVITY</span>
              <span className={styles.metricValue}>
                {formatTimeAgo(session.lastActivityAt)}
              </span>
            </div>
          </div>

          {/* Active Tools */}
          {session.activeTools.length > 0 && (
            <div className={styles.toolsSection}>
              <span className={styles.metricLabel}>ACTIVE TOOLS</span>
              <div className={styles.toolChips}>
                {session.activeTools.map((tool) => (
                  <span key={tool} className={styles.toolChip}>
                    {tool}
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Approval Banner */}
          {session.status === "needs_approval" && session.approvalDetail && (
            <div className={styles.approvalBanner}>
              <div className={styles.approvalTitle}>
                ⚠ Waiting for Approval
              </div>
              <div className={styles.approvalBody}>
                <strong>{session.approvalDetail.tool}:</strong>{" "}
                {session.approvalDetail.description}
              </div>
              <div className={styles.approvalHint}>
                操作は各環境で行ってください
              </div>
            </div>
          )}

        </div>
      )}
    </div>
  );
}
