import type { Environment } from "../../lib/types";

interface EnvironmentIconProps {
  environment: Environment;
  size?: number;
}

const ENVIRONMENT_COLORS: Record<Environment, string> = {
  terminal: "#888888",
  vscode: "#3B82F6",
  cursor: "#06B6D4",
  desktop: "#E8734A",
  web: "#A855F7",
};

function TerminalIcon({ size }: { size: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <polygon points="8,6 18,12 8,18" fill="currentColor" />
    </svg>
  );
}

function VscodeIcon({ size }: { size: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <rect
        x="4"
        y="5"
        width="16"
        height="14"
        rx="2"
        stroke="currentColor"
        strokeWidth="1.8"
        fill="none"
      />
      <polyline
        points="8,10 11,13 8,16"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
      <line
        x1="13"
        y1="16"
        x2="16"
        y2="16"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
      />
    </svg>
  );
}

function CursorIcon({ size }: { size: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path
        d="M6 3L6 18L10 14L15 19L18 16L13 11L18 7Z"
        fill="currentColor"
      />
    </svg>
  );
}

function DesktopIcon({ size }: { size: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path
        d="M12 3C7 3 3 6.5 3 11C3 13.5 4.3 15.7 6.5 17L6 21L10 18.5C10.6 18.7 11.3 18.8 12 18.8C17 18.8 21 15.3 21 10.9C21 6.5 17 3 12 3Z"
        fill="currentColor"
      />
    </svg>
  );
}

function WebIcon({ size }: { size: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <circle
        cx="12"
        cy="12"
        r="9"
        stroke="currentColor"
        strokeWidth="1.8"
        fill="none"
      />
      <ellipse
        cx="12"
        cy="12"
        rx="4"
        ry="9"
        stroke="currentColor"
        strokeWidth="1.8"
        fill="none"
      />
      <line
        x1="3"
        y1="12"
        x2="21"
        y2="12"
        stroke="currentColor"
        strokeWidth="1.8"
      />
    </svg>
  );
}

const ICON_COMPONENTS: Record<
  Environment,
  React.ComponentType<{ size: number }>
> = {
  terminal: TerminalIcon,
  vscode: VscodeIcon,
  cursor: CursorIcon,
  desktop: DesktopIcon,
  web: WebIcon,
};

export function EnvironmentIcon({ environment, size = 32 }: EnvironmentIconProps) {
  const color = ENVIRONMENT_COLORS[environment];
  const IconComponent = ICON_COMPONENTS[environment];
  const iconSize = size * 0.55;

  return (
    <span
      style={{
        width: size,
        height: size,
        borderRadius: "50%",
        backgroundColor: `${color}40`,
        color: color,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        flexShrink: 0,
      }}
    >
      <IconComponent size={iconSize} />
    </span>
  );
}

export { ENVIRONMENT_COLORS };
