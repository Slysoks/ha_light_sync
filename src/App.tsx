import "./App.css";
import { useEffect, useState } from "react";
import { availableMonitors, type Monitor } from "@tauri-apps/api/window";

function App() {
  const [selectedScreen, setSelectedScreen] = useState(0);
  const [monitors, setMonitors] = useState<Monitor[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    availableMonitors()
      .then((available) => {
        if (active) {
          setMonitors(available);
          setSelectedScreen(0);
        }
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(reason instanceof Error ? reason.message : String(reason));
        }
      });

    return () => {
      active = false;
    };
  }, []);

  const sortedMonitors = [...monitors].sort(
    (a, b) => a.position.x - b.position.x,
  );
  const selectedMonitor = sortedMonitors[selectedScreen];

  return (
    <main className="container">
      <h1>Connected monitors</h1>
      {error && <p role="alert">Unable to load monitors: {error}</p>}
      {!error && monitors.length === 0 && <p>No monitors found.</p>}
      <div className="monitor-selector" aria-label="Connected monitors">
        {sortedMonitors.map((monitor, index) => (
          <button
            key={`${monitor.name ?? "monitor"}-${monitor.position.x}-${monitor.position.y}`}
            className={`monitor-button ${selectedScreen === index ? "selected" : ""}`}
            onClick={() => setSelectedScreen(index)}
            type="button"
            style={{
              aspectRatio: `${monitor.size.width} / ${monitor.size.height}`,
              height: "auto",
            }}
          >
            {monitor.name ?? `Monitor ${index + 1}`}
          </button>
        ))}
      </div>
      {selectedMonitor && (
        <div className="screen-display">
          <p>
            {monitors.map((m, i) => (
              <span key={i}>
                {m.name ?? `Monitor ${i + 1}`} ({m.size.width} × {m.size.height})
              </span>
            ))}
          </p>
          <p>Scale factor: {selectedMonitor.scaleFactor}</p>
          <p>
            Position: ({selectedMonitor.position.x}, {selectedMonitor.position.y})
          </p>
        </div>
      )}
    </main>
  );
}

export default App;
