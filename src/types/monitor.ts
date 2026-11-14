import type { Monitor } from "@tauri-apps/api/window";

export type MonitorCapture = {
  position: [number, number];
  data_url: string;
};

export function monitorKey(monitor: Monitor): string {
  return `${monitor.position.x},${monitor.position.y}`;
}

export function capturesByPosition(
  captures: MonitorCapture[],
): Record<string, string> {
  return Object.fromEntries(
    captures.map((capture) => [
      `${capture.position[0]},${capture.position[1]}`,
      capture.data_url,
    ]),
  );
}
