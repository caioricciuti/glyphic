<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import PipelineCanvas from "./PipelineCanvas.svelte";
  import { Plus, Save, Trash2, Play, Square, ChevronDown } from "lucide-svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";

  import type { Node, Edge } from "@xyflow/svelte";

  // Rust backend format
  interface RustPipeline {
    id: string;
    name: string;
    nodes: { id: string; type: string; label: string; x: number; y: number; config: Record<string, string> }[];
    connections: { id: string; from_node: string; to_node: string }[];
    created_at: string;
    updated_at: string;
  }

  type NodeStatus = "idle" | "running" | "done" | "error";

  interface NodeResult {
    nodeId: string;
    label: string;
    output: string;
    status: NodeStatus;
    duration: number;
  }

  let pipelines = $state<RustPipeline[]>([]);
  let activePipelineId = $state<string | null>(null);
  let activePipelineName = $state("New Pipeline");
  let flowNodes = $state<Node[]>([]);
  let flowEdges = $state<Edge[]>([]);

  // Convert Rust → Svelte Flow
  function toFlowFormat(p: RustPipeline): { nodes: Node[]; edges: Edge[] } {
    const nodes: Node[] = p.nodes.map((n) => ({
      id: n.id,
      type: n.type,
      position: { x: n.x, y: n.y },
      data: { label: n.label, config: n.config, status: "idle" },
    }));
    const edges: Edge[] = p.connections.map((c) => ({
      id: c.id,
      source: c.from_node,
      target: c.to_node,
      animated: true,
      type: "smoothstep",
    }));
    return { nodes, edges };
  }

  // Convert Svelte Flow → Rust
  function toRustFormat(): RustPipeline {
    return {
      id: activePipelineId ?? crypto.randomUUID(),
      name: activePipelineName,
      nodes: flowNodes.map((n) => ({
        id: n.id,
        type: n.type ?? "bash",
        label: (n.data as Record<string, string>).label ?? "",
        x: n.position.x,
        y: n.position.y,
        config: (n.data as Record<string, Record<string, string>>).config ?? {},
      })),
      connections: flowEdges.map((e) => ({
        id: e.id,
        from_node: e.source,
        to_node: e.target,
      })),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
  }

  function loadPipeline(p: RustPipeline) {
    activePipelineId = p.id;
    activePipelineName = p.name;
    const { nodes, edges } = toFlowFormat(p);
    flowNodes = nodes;
    flowEdges = edges;
    nodeStatuses = {};
    results = [];
    showResults = false;
  }
  let saving = $state(false);
  let running = $state(false);
  let saveMessage = $state<string | null>(null);
  let deleteDialogOpen = $state(false);
  let showPipelineList = $state(false);
  let nodeStatuses = $state<Record<string, NodeStatus>>({});
  let results = $state<NodeResult[]>([]);
  let showResults = $state(false);

  function createNew() {
    activePipelineId = crypto.randomUUID();
    activePipelineName = "New Pipeline";
    flowNodes = [
      { id: crypto.randomUUID(), type: "input", position: { x: 50, y: 200 }, data: { label: "Start", config: {}, status: "idle" } },
      { id: crypto.randomUUID(), type: "output", position: { x: 600, y: 200 }, data: { label: "End", config: {}, status: "idle" } },
    ];
    flowEdges = [];
    nodeStatuses = {};
    results = [];
    showResults = false;
  }

  async function savePipeline() {
    if (!activePipelineId) return;
    saving = true;
    try {
      const pipeline = toRustFormat();
      await invoke("save_pipeline", { pipeline });
      saveMessage = "Saved!";
      setTimeout(() => (saveMessage = null), 2000);
      await loadPipelines();
    } catch (e) {
      saveMessage = `Error: ${e}`;
    } finally {
      saving = false;
    }
  }

  async function deletePipeline() {
    if (!activePipelineId) return;
    try {
      await invoke("delete_pipeline", { id: activePipelineId });
      activePipelineId = null;
      flowNodes = [];
      flowEdges = [];
      await loadPipelines();
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      deleteDialogOpen = false;
    }
  }

  let eventUnlisten: UnlistenFn | null = null;

  async function runPipeline() {
    if (!activePipelineId) return;
    running = true;
    results = [];
    showResults = true;
    nodeStatuses = {};

    // Listen for pipeline events from Rust background thread
    if (eventUnlisten) eventUnlisten();
    eventUnlisten = await listen<Record<string, string>>("pipeline-event", (event) => {
      const d = event.payload;
      if (d.type === "started") {
        results = [...results, { nodeId: "", label: "Pipeline", output: d.message ?? "Started", status: "done" as NodeStatus, duration: 0 }];
      } else if (d.type === "node_start") {
        nodeStatuses = { ...nodeStatuses, [d.node_id]: "running" };
        results = [...results, { nodeId: d.node_id, label: d.label ?? "", output: "Running...", status: "running" as NodeStatus, duration: 0 }];
      } else if (d.type === "node_done") {
        nodeStatuses = { ...nodeStatuses, [d.node_id]: "done" };
        results = results.filter((r) => !(r.nodeId === d.node_id && r.status === "running"));
        results = [...results, { nodeId: d.node_id, label: d.label ?? "", output: d.output || "(empty)", status: "done" as NodeStatus, duration: parseInt(d.duration ?? "0") }];
      } else if (d.type === "node_error") {
        nodeStatuses = { ...nodeStatuses, [d.node_id]: "error" };
        results = results.filter((r) => !(r.nodeId === d.node_id && r.status === "running"));
        results = [...results, { nodeId: d.node_id, label: d.label ?? "", output: d.output ?? "Error", status: "error" as NodeStatus, duration: parseInt(d.duration ?? "0") }];
      } else if (d.type === "completed" || d.type === "cancelled") {
        results = [...results, { nodeId: "", label: "Pipeline", output: d.message ?? d.type, status: "done" as NodeStatus, duration: 0 }];
        running = false;
      }
    });

    // Fire and forget — Rust runs in a background thread
    try {
      await invoke("start_pipeline_run", { pipeline: toRustFormat() });
    } catch (e) {
      results = [...results, { nodeId: "", label: "Error", output: `${e}`, status: "error" as NodeStatus, duration: 0 }];
      running = false;
    }
  }

  async function cancelPipeline() {
    await invoke("cancel_pipeline_run");
  }

  async function loadPipelines() {
    try {
      pipelines = await invoke<RustPipeline[]>("list_pipelines");
    } catch (e) {
      console.error("Failed:", e);
    }
  }

  onMount(loadPipelines);
  onDestroy(() => { if (eventUnlisten) eventUnlisten(); });
</script>

<ConfirmDialog
  open={deleteDialogOpen}
  title="Delete Pipeline"
  message="This pipeline will be permanently deleted."
  onconfirm={deletePipeline}
  oncancel={() => (deleteDialogOpen = false)}
/>

<div class="flex flex-col h-full">
  <!-- Toolbar -->
  <div class="flex items-center justify-between px-4 py-2 border-b border-border bg-bg-secondary shrink-0">
    <div class="flex items-center gap-2">
      <!-- Pipeline selector -->
      <div class="relative">
        <button
          class="flex items-center gap-2 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md hover:border-border-light"
          onclick={() => (showPipelineList = !showPipelineList)}
        >
          <span class="text-text-primary">{activePipelineId ? activePipelineName : "Select pipeline..."}</span>
          <ChevronDown size={12} class="text-text-muted" />
        </button>
        {#if showPipelineList}
          <button class="fixed inset-0 z-40" onclick={() => (showPipelineList = false)} aria-label="Close"></button>
          <div class="absolute top-full left-0 mt-1 w-64 bg-bg-secondary border border-border rounded-lg shadow-xl z-50 max-h-48 overflow-y-auto">
            {#each pipelines as pipeline}
              <button
                class="w-full text-left px-3 py-2 text-sm hover:bg-bg-hover {activePipelineId === pipeline.id ? 'text-accent' : 'text-text-secondary'}"
                onclick={() => { loadPipeline(pipeline); showPipelineList = false; }}
              >
                {pipeline.name}
                <span class="text-[10px] text-text-muted ml-1">{pipeline.nodes.length} nodes</span>
              </button>
            {/each}
            {#if pipelines.length === 0}
              <p class="px-3 py-2 text-xs text-text-muted">No pipelines yet</p>
            {/if}
          </div>
        {/if}
      </div>

      {#if activePipelineId}
        <input
          type="text"
          class="px-2 py-1 text-sm bg-transparent border-b border-transparent hover:border-border focus:border-accent text-text-primary focus:outline-none"
          bind:value={activePipelineName}
        />
      {/if}

      <button
        class="flex items-center gap-1 px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md"
        onclick={createNew}
      >
        <Plus size={12} />
        New
      </button>
    </div>

    {#if activePipelineId}
      <div class="flex items-center gap-2">
        {#if saveMessage}
          <span class="text-xs {saveMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">{saveMessage}</span>
        {/if}
        {#if running}
          <button
            class="flex items-center gap-1 px-3 py-1.5 text-xs bg-danger/20 text-danger rounded-md hover:bg-danger/30"
            onclick={cancelPipeline}
          >
            <Square size={12} />
            Cancel
          </button>
        {:else}
          <button
            class="flex items-center gap-1 px-3 py-1.5 text-xs bg-success/20 text-success rounded-md hover:bg-success/30"
            onclick={runPipeline}
          >
            <Play size={12} />
            Run
          </button>
        {/if}
        <button
          class="flex items-center gap-1 px-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30"
          onclick={savePipeline}
          disabled={saving}
        >
          <Save size={12} />
          {saving ? "..." : "Save"}
        </button>
        <button
          class="p-1.5 text-text-muted hover:text-danger"
          onclick={() => (deleteDialogOpen = true)}
          aria-label="Delete pipeline"
        >
          <Trash2 size={14} />
        </button>
      </div>
    {/if}
  </div>

  <!-- Canvas -->
  <div class="flex-1 overflow-hidden">
    {#if activePipelineId}
      <div class="flex h-full">
        <div class="flex-1">
          <PipelineCanvas bind:nodes={flowNodes} bind:edges={flowEdges} {nodeStatuses} />
        </div>

        <!-- Results Panel -->
        {#if showResults && results.length > 0}
          <div class="w-80 shrink-0 border-l border-border bg-bg-secondary flex flex-col">
            <div class="flex items-center justify-between px-3 py-2 border-b border-border">
              <h3 class="text-xs font-medium text-text-secondary">Results</h3>
              <button class="text-text-muted hover:text-text-primary text-xs" onclick={() => { showResults = false; nodeStatuses = {}; }}>Clear</button>
            </div>
            <div class="flex-1 overflow-y-auto">
              {#each results as result}
                <div class="px-3 py-2 border-b border-border/50">
                  <div class="flex items-center justify-between mb-1">
                    <span class="text-xs font-medium {result.status === 'done' ? 'text-success' : 'text-danger'}">{result.label}</span>
                    <span class="text-[10px] text-text-muted">{result.duration}ms</span>
                  </div>
                  <pre class="text-[10px] text-text-secondary font-mono whitespace-pre-wrap max-h-32 overflow-y-auto bg-bg-tertiary rounded p-2">{result.output.slice(0, 500)}{result.output.length > 500 ? "..." : ""}</pre>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {:else}
      <div class="flex flex-col items-center justify-center h-full text-text-muted">
        <svg class="w-16 h-16 mb-4 opacity-20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M4 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM14 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1v-4zM14 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
          <path d="M10 7h4M7 10v4M17 10v4" />
        </svg>
        <p class="text-sm">Create a pipeline to get started</p>
        <p class="text-xs mt-1">Visual workflows powered by Claude Code</p>
        <button
          class="mt-4 flex items-center gap-1.5 px-4 py-2 text-sm bg-accent hover:bg-accent-hover text-white rounded-md"
          onclick={createNew}
        >
          <Plus size={14} />
          New Pipeline
        </button>
      </div>
    {/if}
  </div>
</div>
