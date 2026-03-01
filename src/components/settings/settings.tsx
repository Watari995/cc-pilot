import { useSettingsStore } from "../../hooks/use-settings";
import type { IdeApp, TerminalApp } from "../../lib/types";
import styles from "./settings.module.css";

const ACCENT_PRESETS = [
  { color: "#E8734A", label: "Orange" },
  { color: "#22D3EE", label: "Cyan" },
  { color: "#A855F7", label: "Purple" },
  { color: "#22C55E", label: "Green" },
  { color: "#3B82F6", label: "Blue" },
];

const TERMINAL_OPTIONS: { value: TerminalApp; label: string }[] = [
  { value: "ghostty", label: "Ghostty" },
  { value: "iterm2", label: "iTerm2" },
  { value: "terminal", label: "Terminal.app" },
  { value: "wezterm", label: "WezTerm" },
];

const IDE_OPTIONS: { value: IdeApp; label: string }[] = [
  { value: "cursor", label: "Cursor" },
  { value: "vscode", label: "VS Code" },
];

interface SettingsProps {
  onClose: () => void;
}

export function Settings({ onClose }: SettingsProps) {
  const { settings, update } = useSettingsStore();

  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onClose}>
          ←
        </button>
        <span className={styles.pageTitle}>Settings</span>
      </div>

      {/* Accent Color */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>Accent Color</div>
        <div className={styles.colorPicker}>
          {ACCENT_PRESETS.map((preset) => (
            <button
              key={preset.color}
              className={`${styles.colorSwatch} ${
                settings.accentColor.toUpperCase() ===
                preset.color.toUpperCase()
                  ? styles.colorSwatchActive
                  : ""
              }`}
              style={{ backgroundColor: preset.color }}
              title={preset.label}
              onClick={() => update({ accentColor: preset.color })}
            />
          ))}
          <input
            className={styles.colorInput}
            type="text"
            value={settings.accentColor}
            onChange={(e) => {
              const v = e.target.value;
              if (/^#[0-9A-Fa-f]{6}$/.test(v)) {
                update({ accentColor: v });
              }
            }}
            placeholder="#E8734A"
          />
        </div>
      </div>

      {/* Terminal App */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>Applications</div>
        <div className={styles.row}>
          <div>
            <div className={styles.rowLabel}>Terminal App</div>
            <div className={styles.rowHint}>
              Used when jumping to terminal sessions
            </div>
          </div>
          <select
            className={styles.select}
            value={settings.terminalApp}
            onChange={(e) =>
              update({ terminalApp: e.target.value as TerminalApp })
            }
          >
            {TERMINAL_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>
        <div className={styles.row}>
          <div>
            <div className={styles.rowLabel}>IDE</div>
            <div className={styles.rowHint}>
              IDE sessions are detected as this app
            </div>
          </div>
          <select
            className={styles.select}
            value={settings.defaultIde}
            onChange={(e) =>
              update({ defaultIde: e.target.value as IdeApp })
            }
          >
            {IDE_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Notifications */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>Notifications</div>
        <div className={styles.row}>
          <div>
            <div className={styles.rowLabel}>Desktop Notifications</div>
            <div className={styles.rowHint}>
              Notify when a session needs approval
            </div>
          </div>
          <button
            className={`${styles.toggle} ${
              settings.notificationsEnabled ? styles.toggleActive : ""
            }`}
            onClick={() =>
              update({ notificationsEnabled: !settings.notificationsEnabled })
            }
          >
            <span
              className={`${styles.toggleKnob} ${
                settings.notificationsEnabled ? styles.toggleKnobActive : ""
              }`}
            />
          </button>
        </div>
      </div>

      {/* Launch at Login */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>General</div>
        <div className={styles.row}>
          <div>
            <div className={styles.rowLabel}>Launch at Login</div>
            <div className={styles.rowHint}>
              Start cc-pilot when you log in
            </div>
          </div>
          <button
            className={`${styles.toggle} ${
              settings.launchAtLogin ? styles.toggleActive : ""
            }`}
            onClick={() =>
              update({ launchAtLogin: !settings.launchAtLogin })
            }
          >
            <span
              className={`${styles.toggleKnob} ${
                settings.launchAtLogin ? styles.toggleKnobActive : ""
              }`}
            />
          </button>
        </div>
      </div>

      {/* Web Session Key */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>Web Session</div>
        <div className={styles.row}>
          <div>
            <div className={styles.rowLabel}>Session Key</div>
            <div className={styles.rowHint}>
              For future web session monitoring
            </div>
          </div>
          <input
            className={styles.textInput}
            type="password"
            value={settings.claudeSessionKey ?? ""}
            onChange={(e) =>
              update({
                claudeSessionKey: e.target.value || undefined,
              })
            }
            placeholder="Not configured"
          />
        </div>
      </div>
    </div>
  );
}
