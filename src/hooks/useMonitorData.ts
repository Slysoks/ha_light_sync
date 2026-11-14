import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { availableMonitors, type Monitor } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { capturesByPosition, type MonitorCapture } from "../types/monitor";

function errorMessage(reason: unknown): string {
  return reason instanceof Error ? reason.message : String(reason);
}

export function useMonitorData() {
  const [monitors, setMonitors] = useState<Monitor[]>([]);
  const [screenshots, setScreenshots] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    let refreshing = false;

    const refreshMonitors = () => {
      availableMonitors()
        .then((available) => {
          if (active) {
            setMonitors(available);
          }
        })
        .catch((reason: unknown) => {
          if (active) {
            setError(errorMessage(reason));
          }
        });
    };

    const refreshScreenshots = () => {
      if (refreshing) {
        return;
      }

      refreshing = true;
      invoke<MonitorCapture[]>("capture_monitors")
        .then((captures) => {
          if (active) {
            setScreenshots(capturesByPosition(captures));
            setError(null);
          }
        })
        .catch((reason: unknown) => {
          if (active) {
            setError(errorMessage(reason));
          }
        })
        .finally(() => {
          refreshing = false;
        });
    };

    refreshMonitors();
    refreshScreenshots();
    const refreshInterval = window.setInterval(refreshScreenshots, 1000);

    let unlisten: (() => void) | undefined;
    listen<number>("monitors-changed", () => {
      refreshMonitors();
      refreshScreenshots();
    })
      .then((removeListener) => {
        if (active) {
          unlisten = removeListener;
        } else {
          removeListener();
        }
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(`Unable to listen for monitor changes: ${errorMessage(reason)}`);
        }
      });

    return () => {
      active = false;
      window.clearInterval(refreshInterval);
      unlisten?.();
    };
  }, []);

  return { monitors, screenshots, error };
}
