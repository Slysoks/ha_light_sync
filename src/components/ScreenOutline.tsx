import { useEffect } from "react";

export function ScreenOutline() {
  useEffect(() => {
    document.documentElement.classList.add("screen-outline-window");

    return () => {
      document.documentElement.classList.remove("screen-outline-window");
    };
  }, []);

  return <div className="screen-outline" />;
}
