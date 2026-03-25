import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface TerminalSession {
  id: string;
  projectPath: string;
  projectName: string;
  terminal: Terminal;
  fitAddon: FitAddon;
  containerEl: HTMLDivElement;
  alive: boolean;
  unlistenOutput: UnlistenFn | null;
  unlistenExit: UnlistenFn | null;
}

let sessions = $state<TerminalSession[]>([]);
let activeSessionId = $state<string | null>(null);

export function getSessions(): TerminalSession[] {
  return sessions;
}

export function getActiveSessionId(): string | null {
  return activeSessionId;
}

export function getActiveSession(): TerminalSession | undefined {
  return sessions.find((s) => s.id === activeSessionId);
}

export function switchSession(id: string) {
  activeSessionId = id;
}

const THEME = {
  background: "#0d1117",
  foreground: "#e6edf3",
  cursor: "#c084fc",
  selectionBackground: "#c084fc40",
  black: "#0d1117",
  red: "#f85149",
  green: "#3fb950",
  yellow: "#d29922",
  blue: "#58a6ff",
  magenta: "#c084fc",
  cyan: "#39d353",
  white: "#e6edf3",
  brightBlack: "#8b949e",
  brightRed: "#f85149",
  brightGreen: "#3fb950",
  brightYellow: "#d29922",
  brightBlue: "#58a6ff",
  brightMagenta: "#c084fc",
  brightCyan: "#39d353",
  brightWhite: "#ffffff",
};

export async function createSession(projectPath: string, projectName: string): Promise<string> {
  const id = crypto.randomUUID();

  // Create terminal
  const terminal = new Terminal({
    cursorBlink: true,
    fontSize: 13,
    fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace",
    theme: THEME,
    allowProposedApi: true,
  });

  const fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);

  // Create persistent container element
  const containerEl = document.createElement("div");
  containerEl.style.width = "100%";
  containerEl.style.height = "100%";
  containerEl.style.padding = "8px";
  containerEl.style.boxSizing = "border-box";

  // Open terminal into its own container (off-screen for now)
  containerEl.style.position = "absolute";
  containerEl.style.left = "-9999px";
  document.body.appendChild(containerEl);
  terminal.open(containerEl);

  // Listen for PTY output
  const unlistenOutput = await listen<{ id: string; data: string }>("terminal-output", (event) => {
    if (event.payload.id === id) {
      const bytes = Uint8Array.from(atob(event.payload.data), (c) => c.charCodeAt(0));
      terminal.write(bytes);
    }
  });

  // Listen for PTY exit
  const unlistenExit = await listen<{ id: string }>("terminal-exit", (event) => {
    if (event.payload.id === id) {
      terminal.write("\r\n\x1b[90m[Session ended]\x1b[0m\r\n");
      const session = sessions.find((s) => s.id === id);
      if (session) session.alive = false;
    }
  });

  // Send keystrokes to PTY
  terminal.onData((data) => {
    const encoded = btoa(data);
    invoke("write_terminal", { id, data: encoded });
  });

  // Handle resize
  terminal.onResize(({ cols, rows }) => {
    invoke("resize_terminal", { id, cols, rows });
  });

  const session: TerminalSession = {
    id,
    projectPath,
    projectName,
    terminal,
    fitAddon,
    containerEl,
    alive: true,
    unlistenOutput,
    unlistenExit,
  };

  sessions = [...sessions, session];
  activeSessionId = id;

  // Spawn PTY (use default size, will be resized when attached)
  await invoke("spawn_terminal", { id, path: projectPath, cols: 80, rows: 24 });

  return id;
}

export async function closeSession(id: string) {
  const session = sessions.find((s) => s.id === id);
  if (!session) return;

  // Kill PTY
  if (session.alive) {
    await invoke("kill_terminal", { id }).catch(() => {});
  }

  // Cleanup
  session.unlistenOutput?.();
  session.unlistenExit?.();
  session.terminal.dispose();
  session.containerEl.remove();

  sessions = sessions.filter((s) => s.id !== id);

  // Switch to another session or null
  if (activeSessionId === id) {
    activeSessionId = sessions.length > 0 ? sessions[sessions.length - 1].id : null;
  }
}

export function attachToContainer(parentEl: HTMLElement) {
  const session = getActiveSession();
  if (!session) return;

  // Move container element into the visible parent
  if (session.containerEl.parentElement !== parentEl) {
    session.containerEl.style.position = "relative";
    session.containerEl.style.left = "0";
    parentEl.appendChild(session.containerEl);
  }

  // Fit to new size
  requestAnimationFrame(() => {
    session.fitAddon.fit();
  });
}

export function detachFromContainer() {
  const session = getActiveSession();
  if (!session) return;

  // Move off-screen to keep alive
  session.containerEl.style.position = "absolute";
  session.containerEl.style.left = "-9999px";
  document.body.appendChild(session.containerEl);
}

export function fitActiveSession() {
  const session = getActiveSession();
  if (session) {
    session.fitAddon.fit();
  }
}
