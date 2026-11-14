import "./App.css";
import { useEffect, useState } from "react";
import { MonitorButton } from "./components/MonitorButton";
import { ScreenOutline } from "./components/ScreenOutline";
import { useMonitorData } from "./hooks/useMonitorData";
import { monitorKey } from "./types/monitor";

function MonitorDashboard() {
  const { monitors, screenshots, error } = useMonitorData();
  const [selectedScreen, setSelectedScreen] = useState(0);

  useEffect(() => {
    setSelectedScreen(0);
  }, [monitors]);

  const sortedMonitors = [...monitors].sort(
    (a, b) => a.position.x - b.position.x,
  );

  return (
    <>
      <aside className="monitor-sidebar" aria-label="Connected monitors">
        <div className="monitor-selector">
          {sortedMonitors.map((monitor, index) => (
            <MonitorButton
              key={`${monitor.name ?? "monitor"}-${monitorKey(monitor)}`}
              monitor={monitor}
              index={index}
              selected={selectedScreen === index}
              screenshot={screenshots[monitorKey(monitor)]}
              onSelect={() => setSelectedScreen(index)}
            />
          ))}
        </div>
      </aside>
      <main className="container">
        <img
          src={
            screenshots[
              monitorKey(sortedMonitors[selectedScreen] ?? monitors[0])
            ]
          }
          alt={`Screenshot of ${
            sortedMonitors[selectedScreen]?.name ??
            monitors[0]?.name ??
            "monitor"
          }`}
          className="background-image"
        />
        {error && <p role="alert">Unable to load monitors: {error}</p>}
        {!error && monitors.length === 0 && <p>No monitors found.</p>}
      </main>
    </>
  );
}

function App() {
  return window.location.hash === "#screen-outline" ? (
    <ScreenOutline />
  ) : (
    <MonitorDashboard />
  );
}

export default App;
