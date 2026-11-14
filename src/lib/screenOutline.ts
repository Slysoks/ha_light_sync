import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { Monitor } from "@tauri-apps/api/window";

let screenOutline: WebviewWindow | null = null;
let screenOutlineReady: Promise<WebviewWindow> | null = null;
let outlineQueue = Promise.resolve();
let hideOutlineTimeout: ReturnType<typeof setTimeout> | undefined;

function queueOutlineUpdate(update: () => Promise<void>): void {
  outlineQueue = outlineQueue
    .then(update, update)
    .catch((error: unknown) => {
      console.error("Unable to update screen outline:", error);
    });
}

function createScreenOutline(
  position: ReturnType<Monitor["position"]["toLogical"]>,
  size: ReturnType<Monitor["size"]["toLogical"]>,
): void {
  const overlay = new WebviewWindow("screen-outline", {
    url: "/#screen-outline",
    x: position.x,
    y: position.y,
    width: size.width,
    height: size.height,
    transparent: true,
    decorations: false,
    alwaysOnTop: true,
    focus: false,
    focusable: false,
    resizable: false,
    skipTaskbar: true,
    visible: false,
  });

  screenOutline = overlay;
  screenOutlineReady = new Promise<WebviewWindow>((resolve, reject) => {
    void overlay.once("tauri://created", () => resolve(overlay));
    void overlay.once("tauri://error", () =>
      reject(new Error("Unable to create the screen outline window")),
    );
  })
    .then(async (createdOverlay) => {
      await createdOverlay.setIgnoreCursorEvents(true);
      return createdOverlay;
    })
    .catch((error: unknown) => {
      if (screenOutline === overlay) {
        screenOutline = null;
        screenOutlineReady = null;
      }
      throw error;
    });
}

export function showScreenOutline(monitor: Monitor): void {
  if (hideOutlineTimeout !== undefined) {
    clearTimeout(hideOutlineTimeout);
    hideOutlineTimeout = undefined;
  }

  const position = monitor.position.toLogical(monitor.scaleFactor);
  const size = monitor.size.toLogical(monitor.scaleFactor);

  queueOutlineUpdate(async () => {
    if (!screenOutline) {
      createScreenOutline(position, size);
    }

    const overlay = await screenOutlineReady;
    if (!overlay) {
      return;
    }

    await Promise.all([
      overlay.setPosition(position),
      overlay.setSize(size),
      overlay.show(),
    ]);
  });
}

export function hideScreenOutline(): void {
  hideOutlineTimeout = setTimeout(() => {
    hideOutlineTimeout = undefined;
    queueOutlineUpdate(async () => {
      await screenOutline?.hide();
    });
  }, 80);
}
