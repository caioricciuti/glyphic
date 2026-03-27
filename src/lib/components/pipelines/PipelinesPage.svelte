<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import PipelineCanvas from "./PipelineCanvas.svelte";
  import { Plus, Save, Trash2, Play, ChevronDown } from "lucide-svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";

  interface PipelineNode {
    id: string;
    type: string;
    label: string;
    x: number;
    y: number;
    config: Record<string, string>;
  }

  interface PipelineConnection {
    id: string;
    from_node: string;
    to_node: string;
  }

  interface Pipeline {
    id: string;
    name: string;
    nodes: PipelineNode[];
    connections: PipelineConnection[];
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

  let pipelines = $state<Pipeline[]>([]);
  let activePipeline = $state<Pipeline | null>(null);
  let saving = $state(false);
  let running = $state(false);
  let saveMessage = $state<string | null>(null);
  let deleteDialogOpen = $state(false);
  let showPipelineList = $state(false);
  let nodeStatuses = $state<Record<string, NodeStatus>>({});
  let results = $state<NodeResult[]>([]);
  let showResults = $state(false);

  function createNew() {
    const id = crypto.randomUUID();
    const now = new Date().toISOString();
    activePipeline = {
      id,
      name: "New Pipeline",
      nodes: [
        { id: crypto.randomUUID(), type: "input", label: "Start", x: 100, y: 200, config: { variables: "" } },
        { id: crypto.randomUUID(), type: "output", label: "End", x: 700, y: 200, config: {} },
      ],
      connections: [],
      created_at: now,
      updated_at: now,
    };
  }

  async function savePipeline() {
    if (!activePipeline) return;
    saving = true;
    try {
      activePipeline.updated_at = new Date().toISOString();
      await invoke("save_pipeline", { pipeline: activePipeline });
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
    if (!activePipeline) return;
    try {
      await invoke("delete_pipeline", { id: activePipeline.id });
      activePipeline = null;
      await loadPipelines();
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      deleteDialogOpen = false;
    }
  }

  function topologicalSort(nodes: PipelineNode[], connections: PipelineConnection[]): PipelineNode[] {
    const adj = new Map<string, string[]>();
    const inDegree = new Map<string, number>();
    for (const n of nodes) {
      adj.set(n.id, []);
      inDegree.set(n.id, 0);
    }
    for (const c of connections) {
      adj.get(c.from_node)?.push(c.to_node);
      inDegree.set(c.to_node, (inDegree.get(c.to_node) ?? 0) + 1);
    }
    const queue = nodes.filter((n) => (inDegree.get(n.id) ?? 0) === 0).map((n) => n.id);
    const sorted: string[] = [];
    while (queue.length > 0) {
      const id = queue.shift()!;
      sorted.push(id);
      for (const next of adj.get(id) ?? []) {
        const deg = (inDegree.get(next) ?? 1) - 1;
        inDegree.set(next, deg);
        if (deg === 0) queue.push(next);
      }
    }
    return sorted.map((id) => nodes.find((n) => n.id === id)!).filter(Boolean);
  }

  async function runPipeline() {
    if (!activePipeline) return;
    running = true;
    results = [];
    showResults = true;
    nodeStatuses = {};

    const sorted = topologicalSort(activePipeline.nodes, activePipeline.connections);
    let lastOutput = "";

    for (const node of sorted) {
      if (node.type === "input" || node.type === "output") {
        nodeStatuses = { ...nodeStatuses, [node.id]: "done" };
        continue;
      }

      nodeStatuses = { ...nodeStatuses, [node.id]: "running" };
      const start = Date.now();

      try {
        const output = await invoke<string>("run_pipeline_node", {
          nodeType: node.type,
          config: node.config,
          context: lastOutput || null,
        });
        const duration = Date.now() - start;
        lastOutput = output;
        nodeStatuses = { ...nodeStatuses, [node.id]: "done" };
        results = [...results, { nodeId: node.id, label: node.label, output, status: "done", duration }];
      } catch (e) {
        const duration = Date.now() - start;
        nodeStatuses = { ...nodeStatuses, [node.id]: "error" };
        results = [...results, { nodeId: node.id, label: node.label, output: `${e}`, status: "error", duration }];
        break; // Stop on error
      }
    }

    // Mark output node as done
    const outputNode = activePipeline.nodes.find((n) => n.type === "output");
    if (outputNode) nodeStatuses = { ...nodeStatuses, [outputNode.id]: "done" };

    running = false;
  }

  async function loadPipelines() {
    try {
      pipelines = await invoke<Pipeline[]>("list_pipelines");
    } catch (e) {
      console.error("Failed:", e);
    }
  }

  onMount(loadPipelines);
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
          <span class="text-text-primary">{activePipeline?.name ?? "Select pipeline..."}</span>
          <ChevronDown size={12} class="text-text-muted" />
        </button>
        {#if showPipelineList}
          <button class="fixed inset-0 z-40" onclick={() => (showPipelineList = false)} aria-label="Close"></button>
          <div class="absolute top-full left-0 mt-1 w-64 bg-bg-secondary border border-border rounded-lg shadow-xl z-50 max-h-48 overflow-y-auto">
            {#each pipelines as pipeline}
              <button
                class="w-full text-left px-3 py-2 text-sm hover:bg-bg-hover {activePipeline?.id === pipeline.id ? 'text-accent' : 'text-text-secondary'}"
                onclick={() => { activePipeline = pipeline; showPipelineList = false; }}
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

      {#if activePipeline}
        <input
          type="text"
          class="px-2 py-1 text-sm bg-transparent border-b border-transparent hover:border-border focus:border-accent text-text-primary focus:outline-none"
          bind:value={activePipeline.name}
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

    {#if activePipeline}
      <div class="flex items-center gap-2">
        {#if saveMessage}
          <span class="text-xs {saveMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">{saveMessage}</span>
        {/if}
        <button
          class="flex items-center gap-1 px-3 py-1.5 text-xs bg-success/20 text-success rounded-md hover:bg-success/30 disabled:opacity-50"
          onclick={runPipeline}
          disabled={running}
        >
          <Play size={12} />
          {running ? "Running..." : "Run"}
        </button>
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
    {#if activePipeline}
      <div class="flex h-full">
        <div class="flex-1">
          <PipelineCanvas bind:nodes={activePipeline.nodes} bind:connections={activePipeline.connections} {nodeStatuses} />
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
