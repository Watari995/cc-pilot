import type { Environment } from "../../lib/types";
import vscodePng from "../../assets/icons/vscode.png";
import cursorPng from "../../assets/icons/cursor.png";
import claudePng from "../../assets/icons/claude.png";
import terminalPng from "../../assets/icons/terminal.png";
import chromePng from "../../assets/icons/chrome.png";

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

const PNG_ICONS: Record<Environment, string> = {
  vscode: vscodePng,
  cursor: cursorPng,
  desktop: claudePng,
  terminal: terminalPng,
  web: chromePng,
};

export function EnvironmentIcon({ environment, size = 32 }: EnvironmentIconProps) {
  const pngSrc = PNG_ICONS[environment];

  return (
    <img
      src={pngSrc}
      alt={environment}
      width={size}
      height={size}
      draggable={false}
      style={{
        borderRadius: "22%",
        flexShrink: 0,
        pointerEvents: "none",
      }}
    />
  );
}

export { ENVIRONMENT_COLORS };
