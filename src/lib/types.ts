export type Environment = "terminal" | "vscode" | "cursor" | "desktop" | "web";
export type SessionStatus = "working" | "needs_approval" | "idle" | "done" | "error";
export type TerminalApp = "ghostty" | "iterm2" | "terminal" | "wezterm";
export type IdeApp = "cursor" | "vscode";

export interface ApprovalDetail {
  tool: string;
  description: string;
}

export interface Session {
  id: string;
  projectPath: string;
  projectName: string;
  branchName?: string;
  title: string;
  alias?: string;
  environment: Environment;
  status: SessionStatus;
  model?: string;
  inputTokens: number;
  outputTokens: number;
  activeTools: string[];
  startedAt: string;
  lastActivityAt: string;
  approvalDetail?: ApprovalDetail;
  errorMessage?: string;
}

export interface Settings {
  accentColor: string;
  terminalApp: TerminalApp;
  defaultIde: IdeApp;
  launchAtLogin: boolean;
  notificationsEnabled: boolean;
  claudeSessionKey?: string;
}

export const ENVIRONMENT_LABELS: Record<Environment, string> = {
  terminal: "T",
  vscode: "V",
  cursor: "C",
  desktop: "D",
  web: "W",
};

export const STATUS_COLORS: Record<SessionStatus, string> = {
  working: "var(--accent)",
  needs_approval: "#F59E0B",
  idle: "#888888",
  done: "#22C55E",
  error: "#EF4444",
};
