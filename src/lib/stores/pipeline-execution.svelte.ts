import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type NodeStatus = "idle" | "running" | "done" | "error";

export interface NodeResult {
  nodeId: string;
  label: string;
  output: string;
  input: string;
  config: string;
  nodeType: string;
  status: NodeStatus;
  duration: number;
}

interface RustPipelineFormat {
  id: string;
  name: string;
  nodes: { id: string; type: string; label: string; x: number; y: number; config: Record<string, string> }[];
  connections: { id: string; from_node: string; to_node: string }[];
  created_at: string;
  updated_at: string;
  schedule?: string | null;
  schedule_enabled?: boolean;
}

// --- Persistent state (survives navigation) ---
let running = $state(false);
let results = $state<NodeResult[]>([]);
let nodeStatuses = $state<Record<string, NodeStatus>>({});
let interactiveNodeId = $state<string | null>(null);
let interactiveNodeLabel = $state<string | null>(null);

// Terminal state (not $state — plain JS, managed manually)
let interactiveSessionId: string | null = null;
let pipelineTerminal: Terminal | null = null;
let pipelineFitAddon: FitAddon | null = null;
let pipelineTermContainerEl: HTMLDivElement | null = null;
let eventUnlisten: UnlistenFn | null = null;
let unlistenTermOutput: UnlistenFn | null = null;
let unlistenTermExit: UnlistenFn | null = null;
// accumulatedOutput removed — PTY output is raw terminal rendering, not usable for downstream nodes
let pendingSpawn: { sessionId: string; workingDir: string; prompt: string; skipPerms: boolean } | null = null;
let ptySpawned = false;

const TERM_THEME = {
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

// --- Getters (reactive) ---
export function getRunning(): boolean { return running; }
export function getResults(): NodeResult[] { return results; }
export function getNodeStatuses(): Record<string, NodeStatus> { return nodeStatuses; }
export function getInteractiveNodeId(): string | null { return interactiveNodeId; }
export function getInteractiveNodeLabel(): string | null { return interactiveNodeLabel; }
export function hasTerminal(): boolean { return pipelineTermContainerEl !== null; }

// --- Terminal lifecycle ---
function startInteractiveTerminal(sessionId: string, prompt: string, workingDir: string, skipPerms: boolean) {
  if (pipelineTerminal) fullDispose();

  interactiveSessionId = sessionId;

  const term = new Terminal({
    cursorBlink: true,
    fontSize: 11,
    fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace",
    theme: TERM_THEME,
    allowProposedApi: true,
  });
  const fit = new FitAddon();
  term.loadAddon(fit);

  term.onData((data) => {
    const encoded = btoa(data);
    invoke("write_terminal", { id: sessionId, data: encoded }).catch(() => {});
  });

  term.onResize(({ cols, rows }) => {
    invoke("resize_terminal", { id: sessionId, cols, rows }).catch(() => {});
  });

  // Persistent off-screen container
  const containerEl = document.createElement("div");
  containerEl.style.width = "100%";
  containerEl.style.height = "100%";
  containerEl.style.padding = "4px";
  containerEl.style.boxSizing = "border-box";
  containerEl.style.position = "absolute";
  containerEl.style.left = "-9999px";
  document.body.appendChild(containerEl);
  term.open(containerEl);

  pipelineTerminal = term;
  pipelineFitAddon = fit;
  pipelineTermContainerEl = containerEl;

  // Listen for terminal output
  listen<{ id: string; data: string }>("terminal-output", (event) => {
    if (event.payload.id === interactiveSessionId && pipelineTerminal) {
      const bytes = Uint8Array.from(atob(event.payload.data), (c) => c.charCodeAt(0));
      pipelineTerminal.write(bytes);
    }
  }).then((fn) => { unlistenTermOutput = fn; });

  // Listen for terminal exit
  listen<{ id: string }>("terminal-exit", (event) => {
    if (event.payload.id === interactiveSessionId) {
      // PTY output is raw terminal rendering (ANSI, cursor movements, UI chrome) — not clean text.
      // For downstream chaining, use headless mode (interactive=false + dangerouslySkipPermissions).
      invoke("resume_pipeline_node", { output: "(interactive session completed)" }).catch(console.error);
      fullDispose();
    }
  }).then((fn) => { unlistenTermExit = fn; });

  pendingSpawn = { sessionId, workingDir, prompt, skipPerms };
  ptySpawned = false;
}

function fullDispose() {
  unlistenTermOutput?.();
  unlistenTermExit?.();
  unlistenTermOutput = null;
  unlistenTermExit = null;
  pipelineTerminal?.dispose();
  pipelineTerminal = null;
  pipelineFitAddon = null;
  pipelineTermContainerEl?.remove();
  pipelineTermContainerEl = null;
  interactiveNodeId = null;
  interactiveNodeLabel = null;
  interactiveSessionId = null;
  pendingSpawn = null;
  ptySpawned = false;
}

// --- Public API ---

export async function startPipelineRun(pipeline: RustPipelineFormat) {
  running = true;
  results = [];
  nodeStatuses = {};
  fullDispose();

  if (eventUnlisten) eventUnlisten();
  eventUnlisten = await listen<Record<string, string>>("pipeline-event", (event) => {
    const d = event.payload;
    if (d.type === "started") {
      results = [...results, { nodeId: "", label: "Pipeline", output: d.message ?? "Started", input: "", config: "", nodeType: "", status: "done" as NodeStatus, duration: 0 }];
    } else if (d.type === "node_interactive_start") {
      interactiveNodeId = d.node_id;
      interactiveNodeLabel = d.label ?? "Claude";
      nodeStatuses = { ...nodeStatuses, [d.node_id]: "running" };
      results = results.filter((r) => !(r.nodeId === d.node_id && r.status === "running"));
      results = [...results, { nodeId: d.node_id, label: d.label ?? "", output: "Interactive — waiting for user...", input: "", config: "", nodeType: "claude", status: "running" as NodeStatus, duration: 0 }];
      startInteractiveTerminal(d.session_id, d.prompt, d.working_dir, d.dangerously_skip_permissions === "true");
    } else if (d.type === "node_start") {
      nodeStatuses = { ...nodeStatuses, [d.node_id]: "running" };
      results = [...results, { nodeId: d.node_id, label: d.label ?? "", output: "Running...", input: d.input ?? "", config: d.config ?? "", nodeType: d.node_type ?? "", status: "running" as NodeStatus, duration: 0 }];
    } else if (d.type === "node_done") {
      nodeStatuses = { ...nodeStatuses, [d.node_id]: "done" };
      const idx = results.findIndex((r) => r.nodeId === d.node_id && r.status === "running");
      if (idx >= 0) {
        const prev = results[idx];
        results = [...results.slice(0, idx), { ...prev, output: d.output || "(empty)", status: "done" as NodeStatus, duration: parseInt(d.duration ?? "0") }, ...results.slice(idx + 1)];
      } else {
        results = [...results, { nodeId: d.node_id, label: d.label ?? "", output: d.output || "(empty)", input: "", config: "", nodeType: "", status: "done" as NodeStatus, duration: parseInt(d.duration ?? "0") }];
      }
    } else if (d.type === "node_error") {
      nodeStatuses = { ...nodeStatuses, [d.node_id]: "error" };
      const idx = results.findIndex((r) => r.nodeId === d.node_id && r.status === "running");
      if (idx >= 0) {
        const prev = results[idx];
        results = [...results.slice(0, idx), { ...prev, output: d.output ?? "Error", status: "error" as NodeStatus, duration: parseInt(d.duration ?? "0") }, ...results.slice(idx + 1)];
      } else {
        results = [...results, { nodeId: d.node_id, label: d.label ?? "", output: d.output ?? "Error", input: "", config: "", nodeType: "", status: "error" as NodeStatus, duration: parseInt(d.duration ?? "0") }];
      }
      fullDispose();
    } else if (d.type === "completed" || d.type === "cancelled") {
      results = [...results, { nodeId: "", label: "Pipeline", output: d.message ?? d.type, input: "", config: "", nodeType: "", status: "done" as NodeStatus, duration: 0 }];
      running = false;
      fullDispose();
      if (eventUnlisten) { eventUnlisten(); eventUnlisten = null; }
    }
  });

  try {
    await invoke("start_pipeline_run", { pipeline });
  } catch (e) {
    results = [...results, { nodeId: "", label: "Error", output: `${e}`, input: "", config: "", nodeType: "", status: "error" as NodeStatus, duration: 0 }];
    running = false;
    fullDispose();
  }
}

export async function cancelPipelineRun() {
  await invoke("cancel_pipeline_run");
}

export function clearResults() {
  results = [];
  nodeStatuses = {};
}

export function attachTerminal(parentEl: HTMLDivElement) {
  if (pipelineTermContainerEl) {
    pipelineTermContainerEl.style.position = "relative";
    pipelineTermContainerEl.style.left = "0";
    parentEl.appendChild(pipelineTermContainerEl);
    requestAnimationFrame(() => {
      pipelineFitAddon?.fit();
      // Spawn PTY AFTER fit so it gets the real column width
      if (pendingSpawn && !ptySpawned) {
        ptySpawned = true;
        const { sessionId, workingDir, prompt, skipPerms } = pendingSpawn;
        const cols = pipelineTerminal?.cols ?? 40;
        const rows = pipelineTerminal?.rows ?? 20;
        invoke("spawn_terminal", {
          id: sessionId,
          path: workingDir,
          cols,
          rows,
          prompt,
          dangerouslySkipPermissions: skipPerms || null,
        }).catch(console.error);
        pendingSpawn = null;
      }
    });
  }
}

export function detachTerminal() {
  if (pipelineTermContainerEl) {
    pipelineTermContainerEl.style.position = "absolute";
    pipelineTermContainerEl.style.left = "-9999px";
    document.body.appendChild(pipelineTermContainerEl);
  }
}
