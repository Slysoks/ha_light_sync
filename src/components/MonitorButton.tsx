import type { Monitor } from "@tauri-apps/api/window";
import { hideScreenOutline, showScreenOutline } from "../lib/screenOutline";

type MonitorButtonProps = {
  monitor: Monitor;
  index: number;
  selected: boolean;
  screenshot?: string;
  onSelect: () => void;
};

export function MonitorButton({
  monitor,
  index,
  selected,
  screenshot,
  onSelect,
}: MonitorButtonProps) {
  return (
    <button
      className={`monitor-button ${selected ? "selected" : ""}`}
      onClick={onSelect}
      onMouseEnter={() => showScreenOutline(monitor)}
      onMouseLeave={hideScreenOutline}
      aria-pressed={selected}
      type="button"
      style={{
        aspectRatio: `${monitor.size.width} / ${monitor.size.height}`,
        height: "auto",
      }}
    >
      {screenshot && <img src={screenshot} alt="" className="monitor-preview" />}
      <div className="monitor-info">
        <span>{monitor.name ?? `Monitor ${index + 1}`}</span>
        <span>
          {monitor.size.width} x {monitor.size.height}
        </span>
      </div>
    </button>
  );
}
