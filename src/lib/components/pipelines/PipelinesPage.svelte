<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import PipelineCanvas from "./PipelineCanvas.svelte";
  import CodeEditor from "./CodeEditor.svelte";
  import {
    Plus, Save, Trash2, Play, Square, ChevronDown,
    X, Bot, Terminal, GitBranch, Globe, Repeat, Clock,
    CalendarClock, Plus as PlusIcon, Minus,
    GitCommitHorizontal, Filter, FileInput, FileOutput, Bell, Braces,
  } from "lucide-svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import {
    getRunning, getResults, getNodeStatuses, getInteractiveNodeId, getInteractiveNodeLabel,
    startPipelineRun, cancelPipelineRun, clearResults, attachTerminal, detachTerminal,
  } from "$lib/stores/pipeline-execution.svelte";

  import type { Node, Edge } from "@xyflow/svelte";

  // Rust backend format
  interface RustPipeline {
    id: string;
    name: string;
    nodes: { id: string; type: string; label: string; x: number; y: number; config: Record<string, string> }[];
    connections: { id: string; from_node: string; to_node: string }[];
    created_at: string;
    updated_at: string;
    schedule?: string | null;
    schedule_enabled?: boolean;
  }

  // --- State ---
  let pipelines = $state<RustPipeline[]>([]);
  let activePipelineId = $state<string | null>(null);
  let activePipelineName = $state("New Pipeline");
  let flowNodes = $state<Node[]>([]);
  let flowEdges = $state<Edge[]>([]);
  let saving = $state(false);
  let saveMessage = $state<string | null>(null);
  let deleteDialogOpen = $state(false);
  let showPipelineList = $state(false);
  let selectedNode = $state<Node | null>(null);
  let rightTab = $state<"config" | "results" | "logs">("config");
  let highlightedResultId = $state<string | null>(null);

  // Execution state from persistent store (survives navigation)
  const running = $derived(getRunning());
  const results = $derived(getResults());
  const nodeStatuses = $derived(getNodeStatuses());
  const interactiveNodeId = $derived(getInteractiveNodeId());
  const interactiveNodeLabel = $derived(getInteractiveNodeLabel());

  // Schedule state
  let showSchedulePopover = $state(false);
  let scheduleExpr = $state("");
  let scheduleEnabled = $state(false);
  let scheduleLogs = $state<{ filename: string; timestamp: string; content: string }[]>([]);

  const CRON_PRESETS = [
    { label: "Every 5 minutes", value: "*/5 * * * *" },
    { label: "Every 15 minutes", value: "*/15 * * * *" },
    { label: "Every hour", value: "0 * * * *" },
    { label: "Every 6 hours", value: "0 */6 * * *" },
    { label: "Daily at midnight", value: "0 0 * * *" },
    { label: "Daily at 9am", value: "0 9 * * *" },
    { label: "Weekly (Mon 9am)", value: "0 9 * * 1" },
  ];

  // Svelte action: attach/detach terminal from store
  function attachTerminalAction(node: HTMLDivElement) {
    attachTerminal(node);
    return {
      destroy() { detachTerminal(); }
    };
  }

  // Single node test
  let testingNode = $state(false);
  let testResult = $state<{ output: string; error: boolean } | null>(null);

  async function testSelectedNode() {
    if (!selectedNode) return;
    testingNode = true;
    testResult = null;
    try {
      const node = {
        id: selectedNode.id,
        type: selectedNode.type ?? "bash",
        label: (selectedNode.data as Record<string, string>).label ?? "",
        x: 0, y: 0,
        config: (selectedNode.data as Record<string, Record<string, string>>).config ?? {},
      };
      const output = await invoke<string>("run_single_node", { node, context: null });
      testResult = { output, error: false };
    } catch (e) {
      testResult = { output: `${e}`, error: true };
    } finally {
      testingNode = false;
    }
  }

  // Node icons lookup
  const NODE_ICONS: Record<string, { icon: typeof Bot; color: string }> = {
    claude: { icon: Bot, color: "text-accent" },
    bash: { icon: Terminal, color: "text-success" },
    github: { icon: GitBranch, color: "text-info" },
    http: { icon: Globe, color: "text-danger" },
    transform: { icon: Repeat, color: "text-warning" },
    delay: { icon: Clock, color: "text-text-muted" },
    git: { icon: GitCommitHorizontal, color: "text-info" },
    filter: { icon: Filter, color: "text-accent" },
    readfile: { icon: FileInput, color: "text-info" },
    writefile: { icon: FileOutput, color: "text-warning" },
    notification: { icon: Bell, color: "text-warning" },
    jsonextract: { icon: Braces, color: "text-accent" },
  };

  // --- Format conversions ---
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

  // --- Node config helpers ---
  function updateNodeData(nodeId: string, updates: Record<string, unknown>) {
    flowNodes = flowNodes.map((n) =>
      n.id === nodeId ? { ...n, data: { ...n.data, ...updates } } : n,
    );
    if (selectedNode?.id === nodeId) {
      selectedNode = flowNodes.find((n) => n.id === nodeId) ?? null;
    }
  }

  function updateNodeConfig(nodeId: string, key: string, value: string) {
    const node = flowNodes.find((n) => n.id === nodeId);
    if (!node) return;
    const newConfig = { ...(node.data.config ?? {}), [key]: value };
    updateNodeData(nodeId, { config: newConfig });
  }

  function deleteSelectedNode() {
    if (!selectedNode) return;
    const id = selectedNode.id;
    flowNodes = flowNodes.filter((n) => n.id !== id);
    flowEdges = flowEdges.filter((e) => e.source !== id && e.target !== id);
    selectedNode = null;
  }

  function onSelectNode(node: Node | null) {
    selectedNode = node;
    if (node) {
      // If this node has a completed result, jump to its trace
      const hasResult = results.some((r) => r.nodeId === node.id && (r.status === "done" || r.status === "error"));
      if (hasResult && !running) {
        rightTab = "results";
        highlightedResultId = node.id;
        setTimeout(() => { highlightedResultId = null; }, 2000);
      } else if (node.type !== "input" && node.type !== "output") {
        rightTab = "config";
      }
    }
  }

  // --- HTTP headers helpers ---
  function getHeaders(node: Node): { key: string; value: string }[] {
    try {
      const raw = (node.data as Record<string, Record<string, string>>).config?.headers;
      return raw ? JSON.parse(raw) : [];
    } catch { return []; }
  }

  function setHeaders(nodeId: string, headers: { key: string; value: string }[]) {
    updateNodeConfig(nodeId, "headers", JSON.stringify(headers));
  }

  function addHeader(nodeId: string) {
    const headers = selectedNode ? getHeaders(selectedNode) : [];
    headers.push({ key: "", value: "" });
    setHeaders(nodeId, headers);
  }

  function removeHeader(nodeId: string, idx: number) {
    const headers = selectedNode ? getHeaders(selectedNode) : [];
    headers.splice(idx, 1);
    setHeaders(nodeId, headers);
  }

  function updateHeader(nodeId: string, idx: number, field: "key" | "value", val: string) {
    const headers = selectedNode ? getHeaders(selectedNode) : [];
    if (headers[idx]) {
      headers[idx][field] = val;
      setHeaders(nodeId, headers);
    }
  }

  // --- Pipeline CRUD ---
  function loadPipeline(p: RustPipeline) {
    activePipelineId = p.id;
    activePipelineName = p.name;
    const { nodes, edges } = toFlowFormat(p);
    flowNodes = nodes;
    flowEdges = edges;
    clearResults();
    selectedNode = null;
    scheduleExpr = p.schedule ?? "";
    scheduleEnabled = p.schedule_enabled ?? false;
    scheduleLogs = [];
  }

  function createNew() {
    activePipelineId = crypto.randomUUID();
    activePipelineName = "New Pipeline";
    flowNodes = [
      { id: crypto.randomUUID(), type: "input", position: { x: 50, y: 200 }, data: { label: "Start", config: {}, status: "idle" } },
      { id: crypto.randomUUID(), type: "output", position: { x: 600, y: 200 }, data: { label: "End", config: {}, status: "idle" } },
    ];
    flowEdges = [];
    clearResults();
    selectedNode = null;
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
      selectedNode = null;
      await loadPipelines();
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      deleteDialogOpen = false;
    }
  }

  // --- Run pipeline (delegates to persistent store) ---
  async function runPipeline() {
    if (!activePipelineId) return;
    rightTab = "results";
    await startPipelineRun(toRustFormat());
  }

  async function cancelPipeline() {
    await cancelPipelineRun();
  }

  async function enableSchedule() {
    if (!activePipelineId || !scheduleExpr.trim()) return;
    // Save pipeline first so the schedule script has current data
    await savePipeline();
    try {
      await invoke("enable_pipeline_schedule", { pipelineId: activePipelineId, cronExpr: scheduleExpr.trim() });
      scheduleEnabled = true;
      showSchedulePopover = false;
      await loadPipelines();
    } catch (e) {
      console.error("Schedule failed:", e);
    }
  }

  async function disableSchedule() {
    if (!activePipelineId) return;
    try {
      await invoke("disable_pipeline_schedule", { pipelineId: activePipelineId });
      scheduleEnabled = false;
      await loadPipelines();
    } catch (e) {
      console.error("Disable schedule failed:", e);
    }
  }

  async function loadScheduleLogs() {
    if (!activePipelineId) return;
    try {
      scheduleLogs = await invoke<typeof scheduleLogs>("list_pipeline_logs", { pipelineId: activePipelineId });
      rightTab = "logs";
    } catch (e) {
      console.error("Load logs failed:", e);
    }
  }

  async function loadPipelines() {
    try {
      pipelines = await invoke<RustPipeline[]>("list_pipelines");
    } catch (e) {
      console.error("Failed:", e);
    }
  }

  onMount(() => {
    loadPipelines();
    // If pipeline was running while we were on another page, show results
    if (running) rightTab = "results";
  });
  onDestroy(() => {
    // Only detach terminal — don't kill it. Store keeps everything alive.
    detachTerminal();
  });

  // Derived: whether right panel should show
  const showRightPanel = $derived(
    (selectedNode && selectedNode.type !== "input" && selectedNode.type !== "output") ||
    results.length > 0 ||
    interactiveNodeId !== null ||
    rightTab === "logs"
  );

  // Derived: selected node config shorthand
  const nodeConfig = $derived(
    selectedNode ? ((selectedNode.data as Record<string, Record<string, string>>).config ?? {}) : {}
  );

  const nodeType = $derived(selectedNode?.type ?? "");

  const httpMethod = $derived(nodeConfig.method ?? "GET");
  const showBody = $derived(["POST", "PUT", "PATCH"].includes(httpMethod));
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
                <div class="flex items-center gap-2">
                  <span>{pipeline.name}</span>
                  {#if pipeline.schedule_enabled}
                    <CalendarClock size={10} class="text-accent" />
                  {/if}
                </div>
                <span class="text-[10px] text-text-muted">{pipeline.nodes.length} nodes</span>
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
        <!-- Schedule -->
        <div class="relative">
          <button
            class="flex items-center gap-1 px-3 py-1.5 text-xs rounded-md {scheduleEnabled ? 'bg-accent/20 text-accent' : 'bg-bg-tertiary border border-border text-text-secondary hover:border-accent/30'}"
            onclick={() => (showSchedulePopover = !showSchedulePopover)}
            title={scheduleEnabled ? `Scheduled: ${scheduleExpr}` : "Schedule pipeline"}
          >
            <CalendarClock size={12} />
            {scheduleEnabled ? "Scheduled" : "Schedule"}
          </button>

          {#if showSchedulePopover}
            <button class="fixed inset-0 z-40" onclick={() => (showSchedulePopover = false)} aria-label="Close"></button>
            <div class="absolute top-full right-0 mt-1 w-72 bg-bg-secondary border border-border rounded-lg shadow-xl z-50 p-3 space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-xs font-medium text-text-primary">Pipeline Schedule</span>
                {#if scheduleEnabled}
                  <span class="text-[10px] px-1.5 py-0.5 bg-success/20 text-success rounded">Active</span>
                {/if}
              </div>

              <!-- Presets -->
              <div class="flex flex-wrap gap-1">
                {#each CRON_PRESETS as preset}
                  <button
                    class="px-2 py-1 text-[10px] rounded {scheduleExpr === preset.value ? 'bg-accent text-white' : 'bg-bg-tertiary text-text-secondary hover:bg-bg-hover'}"
                    onclick={() => (scheduleExpr = preset.value)}
                  >
                    {preset.label}
                  </button>
                {/each}
              </div>

              <!-- Custom cron input -->
              <div>
                <label class="text-[10px] text-text-muted">Cron expression</label>
                <input
                  type="text"
                  class="w-full mt-0.5 px-2 py-1.5 text-xs bg-bg-tertiary border border-border rounded text-text-primary font-mono focus:outline-none focus:border-accent"
                  placeholder="*/5 * * * *"
                  bind:value={scheduleExpr}
                />
                <p class="text-[9px] text-text-muted mt-0.5">min hour day month weekday</p>
              </div>

              <!-- Actions -->
              <div class="flex items-center gap-2">
                {#if scheduleEnabled}
                  <button
                    class="flex-1 px-3 py-1.5 text-xs bg-danger/20 text-danger rounded hover:bg-danger/30"
                    onclick={disableSchedule}
                  >
                    Disable
                  </button>
                  <button
                    class="flex-1 px-3 py-1.5 text-xs bg-accent/20 text-accent rounded hover:bg-accent/30"
                    onclick={enableSchedule}
                  >
                    Update
                  </button>
                {:else}
                  <button
                    class="flex-1 px-3 py-1.5 text-xs bg-accent text-white rounded hover:bg-accent-hover disabled:opacity-50"
                    onclick={enableSchedule}
                    disabled={!scheduleExpr.trim()}
                  >
                    Enable Schedule
                  </button>
                {/if}
              </div>

              <!-- View logs link -->
              <button
                class="w-full text-[10px] text-text-muted hover:text-accent text-center"
                onclick={() => { loadScheduleLogs(); showSchedulePopover = false; }}
              >
                View schedule logs
              </button>
            </div>
          {/if}
        </div>

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

  <!-- Canvas + Right Panel -->
  <div class="flex-1 overflow-hidden">
    {#if activePipelineId}
      <div class="flex h-full">
        <!-- Canvas -->
        <div class="flex-1">
          <PipelineCanvas
            bind:nodes={flowNodes}
            bind:edges={flowEdges}
            {nodeStatuses}
            onselectnode={onSelectNode}
          />
        </div>

        <!-- Right Panel (Config + Results) -->
        {#if showRightPanel}
          <div class="w-96 shrink-0 border-l border-border bg-bg-secondary flex flex-col">
            <!-- Tabs -->
            <div class="flex border-b border-border shrink-0">
              {#if selectedNode && selectedNode.type !== "input" && selectedNode.type !== "output"}
                <button
                  class="flex-1 px-3 py-2 text-xs font-medium transition-colors {rightTab === 'config' ? 'text-accent border-b-2 border-accent' : 'text-text-muted hover:text-text-secondary'}"
                  onclick={() => (rightTab = "config")}
                >
                  Config
                </button>
              {/if}
              <button
                class="flex-1 px-3 py-2 text-xs font-medium transition-colors {rightTab === 'results' ? 'text-accent border-b-2 border-accent' : 'text-text-muted hover:text-text-secondary'}"
                onclick={() => (rightTab = "results")}
              >
                Results {results.length > 0 ? `(${results.length})` : ""}
              </button>
              <button
                class="flex-1 px-3 py-2 text-xs font-medium transition-colors {rightTab === 'logs' ? 'text-accent border-b-2 border-accent' : 'text-text-muted hover:text-text-secondary'}"
                onclick={() => { rightTab = "logs"; loadScheduleLogs(); }}
              >
                Logs {scheduleLogs.length > 0 ? `(${scheduleLogs.length})` : ""}
              </button>
            </div>

            <!-- Config Tab -->
            {#if rightTab === "config" && selectedNode && selectedNode.type !== "input" && selectedNode.type !== "output"}
              <div class="flex-1 overflow-y-auto p-4 space-y-4">
                <!-- Node header -->
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    {#if NODE_ICONS[nodeType]}
                      {@const IconComp = NODE_ICONS[nodeType].icon}
                      <div class="w-7 h-7 rounded-lg bg-bg-tertiary flex items-center justify-center">
                        <IconComp size={14} class={NODE_ICONS[nodeType].color} />
                      </div>
                    {/if}
                    <input
                      type="text"
                      class="text-sm font-medium bg-transparent border-b border-transparent hover:border-border focus:border-accent text-text-primary focus:outline-none"
                      value={((selectedNode as any).data.label) ?? ""}
                      oninput={(e) => updateNodeData(selectedNode!.id, { label: (e.target as HTMLInputElement).value })}
                    />
                  </div>
                  <div class="flex items-center gap-1">
                    <button
                      class="flex items-center gap-1 px-2 py-1 text-[10px] rounded {testingNode ? 'bg-warning/20 text-warning' : 'bg-success/20 text-success hover:bg-success/30'}"
                      onclick={testSelectedNode}
                      disabled={testingNode}
                      title="Test this node"
                    >
                      <Play size={10} />
                      {testingNode ? "..." : "Test"}
                    </button>
                    <button
                      class="p-1.5 text-text-muted hover:text-danger rounded"
                      onclick={deleteSelectedNode}
                      aria-label="Delete node"
                      title="Delete node"
                    >
                      <Trash2 size={14} />
                    </button>
                    <button
                      class="p-1.5 text-text-muted hover:text-text-primary rounded"
                      onclick={() => { selectedNode = null; testResult = null; }}
                      aria-label="Close"
                    >
                      <X size={14} />
                    </button>
                  </div>
                </div>

                <!-- Claude Prompt -->
                {#if nodeType === "claude"}
                  <div class="space-y-3">
                    <div>
                      <label class="text-xs text-text-muted font-medium">Prompt</label>
                      <CodeEditor
                        value={nodeConfig.prompt ?? ""}
                        language="text"
                        placeholder="What should Claude do?"
                        minHeight="120px"
                        maxHeight="400px"
                        onchange={(v) => updateNodeConfig(selectedNode!.id, "prompt", v)}
                      />
                      <p class="text-[10px] text-text-muted mt-1">Previous output is automatically passed as context. Use {"{{"}NodeName{"}}"} for specific nodes.</p>
                    </div>

                    <label class="flex items-center gap-2">
                      <input
                        type="checkbox"
                        class="rounded border-border accent-accent"
                        checked={nodeConfig.interactive !== "false"}
                        onchange={(e) => updateNodeConfig(selectedNode!.id, "interactive", (e.target as HTMLInputElement).checked ? "true" : "false")}
                      />
                      <span class="text-xs text-text-muted">Interactive mode</span>
                    </label>
                    <p class="text-[10px] text-text-muted -mt-2 ml-5">Runs Claude in a terminal — you can grant permissions and answer questions. When off, uses headless <code class="text-accent">--print</code> mode.</p>

                    <label class="flex items-center gap-2">
                      <input
                        type="checkbox"
                        class="rounded border-border accent-accent"
                        checked={nodeConfig.dangerouslySkipPermissions === "true"}
                        onchange={(e) => updateNodeConfig(selectedNode!.id, "dangerouslySkipPermissions", (e.target as HTMLInputElement).checked ? "true" : "false")}
                      />
                      <span class="text-xs text-text-muted">Auto-accept all permissions</span>
                    </label>
                    <p class="text-[10px] text-text-muted -mt-2 ml-5">Adds <code class="text-accent">--dangerously-skip-permissions</code>. Claude won't ask for confirmation. Use only in safe environments.</p>

                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">Working Directory</span>
                      <input
                        type="text"
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
                        placeholder="~/projects/my-app (default: home)"
                        value={nodeConfig.path ?? ""}
                        oninput={(e) => updateNodeConfig(selectedNode!.id, "path", (e.target as HTMLInputElement).value)}
                      />
                    </label>
                  </div>

                <!-- Bash / GitHub -->
                {:else if nodeType === "bash" || nodeType === "github"}
                  <div class="space-y-2">
                    <label class="text-xs text-text-muted font-medium">{nodeType === "bash" ? "Command" : "GitHub CLI Command"}</label>
                    <CodeEditor
                      value={nodeConfig.command ?? ""}
                      language="shell"
                      placeholder={nodeType === "bash" ? 'echo "Hello World"' : "gh pr list --limit 10"}
                      minHeight="80px"
                      maxHeight="400px"
                      onchange={(v) => updateNodeConfig(selectedNode!.id, "command", v)}
                    />
                    <p class="text-[10px] text-text-muted">Use {"{{"}input{"}}"} for previous output or {"{{"}NodeName{"}}"} for specific node</p>
                  </div>

                <!-- HTTP Request -->
                {:else if nodeType === "http"}
                  <div class="space-y-4">
                    <!-- Method + URL -->
                    <div class="grid grid-cols-4 gap-2">
                      <label class="block">
                        <span class="text-xs text-text-muted font-medium">Method</span>
                        <select
                          class="w-full mt-1 px-2 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                          value={nodeConfig.method ?? "GET"}
                          onchange={(e) => updateNodeConfig(selectedNode!.id, "method", (e.target as HTMLSelectElement).value)}
                        >
                          <option value="GET">GET</option>
                          <option value="POST">POST</option>
                          <option value="PUT">PUT</option>
                          <option value="PATCH">PATCH</option>
                          <option value="DELETE">DELETE</option>
                          <option value="HEAD">HEAD</option>
                        </select>
                      </label>
                      <label class="block col-span-3">
                        <span class="text-xs text-text-muted font-medium">URL</span>
                        <input
                          type="text"
                          class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
                          placeholder="https://api.example.com/endpoint"
                          value={nodeConfig.url ?? ""}
                          oninput={(e) => updateNodeConfig(selectedNode!.id, "url", (e.target as HTMLInputElement).value)}
                        />
                      </label>
                    </div>

                    <!-- Headers -->
                    <div class="space-y-2">
                      <div class="flex items-center justify-between">
                        <span class="text-xs text-text-muted font-medium">Headers</span>
                        <button
                          class="flex items-center gap-1 px-2 py-0.5 text-[10px] text-accent hover:text-accent-hover rounded"
                          onclick={() => addHeader(selectedNode!.id)}
                        >
                          <PlusIcon size={10} />
                          Add
                        </button>
                      </div>
                      {#each getHeaders(selectedNode!) as header, idx}
                        <div class="flex items-center gap-1">
                          <input
                            type="text"
                            class="flex-1 px-2 py-1.5 text-xs bg-bg-tertiary border border-border rounded text-text-primary font-mono focus:outline-none focus:border-accent"
                            placeholder="Key"
                            value={header.key}
                            oninput={(e) => updateHeader(selectedNode!.id, idx, "key", (e.target as HTMLInputElement).value)}
                          />
                          <input
                            type="text"
                            class="flex-1 px-2 py-1.5 text-xs bg-bg-tertiary border border-border rounded text-text-primary font-mono focus:outline-none focus:border-accent"
                            placeholder="Value"
                            value={header.value}
                            oninput={(e) => updateHeader(selectedNode!.id, idx, "value", (e.target as HTMLInputElement).value)}
                          />
                          <button
                            class="p-1 text-text-muted hover:text-danger rounded shrink-0"
                            onclick={() => removeHeader(selectedNode!.id, idx)}
                          >
                            <Minus size={12} />
                          </button>
                        </div>
                      {/each}
                    </div>

                    <!-- Body (for POST/PUT/PATCH) -->
                    {#if showBody}
                      <div class="space-y-2">
                        <label class="text-xs text-text-muted font-medium">Body</label>
                        <CodeEditor
                          value={nodeConfig.body ?? ""}
                          language="json"
                          placeholder={'{"key": "value"}'}
                          minHeight="80px"
                          maxHeight="300px"
                          onchange={(v) => updateNodeConfig(selectedNode!.id, "body", v)}
                        />
                      </div>
                    {/if}

                    <p class="text-[10px] text-text-muted">Use {"{{"}input{"}}"} or {"{{"}NodeName{"}}"} in URL, headers, or body.</p>
                  </div>

                <!-- Transform -->
                {:else if nodeType === "transform"}
                  <label class="block">
                    <span class="text-xs text-text-muted font-medium">Operation</span>
                    <select
                      class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                      value={nodeConfig.operation ?? "passthrough"}
                      onchange={(e) => updateNodeConfig(selectedNode!.id, "operation", (e.target as HTMLSelectElement).value)}
                    >
                      <option value="passthrough">Pass through</option>
                      <option value="uppercase">Uppercase</option>
                      <option value="lowercase">Lowercase</option>
                      <option value="trim">Trim whitespace</option>
                      <option value="line_count">Count lines</option>
                      <option value="word_count">Count words</option>
                      <option value="first_line">First line only</option>
                      <option value="json_pretty">JSON pretty print</option>
                    </select>
                  </label>

                <!-- Delay -->
                {:else if nodeType === "delay"}
                  <label class="block">
                    <span class="text-xs text-text-muted font-medium">Seconds</span>
                    <input
                      type="number"
                      class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                      placeholder="1"
                      value={nodeConfig.seconds ?? "1"}
                      oninput={(e) => updateNodeConfig(selectedNode!.id, "seconds", (e.target as HTMLInputElement).value)}
                    />
                  </label>

                <!-- Git -->
                {:else if nodeType === "git"}
                  <div class="space-y-3">
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">Repository Path</span>
                      <input
                        type="text"
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
                        placeholder="/path/to/repo"
                        value={nodeConfig.path ?? ""}
                        oninput={(e) => updateNodeConfig(selectedNode!.id, "path", (e.target as HTMLInputElement).value)}
                      />
                    </label>
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">Operation</span>
                      <select
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                        value={nodeConfig.operation ?? "status"}
                        onchange={(e) => updateNodeConfig(selectedNode!.id, "operation", (e.target as HTMLSelectElement).value)}
                      >
                        <option value="status">Status</option>
                        <option value="log">Log</option>
                        <option value="diff">Diff</option>
                        <option value="pull">Pull</option>
                        <option value="push">Push</option>
                        <option value="commit">Commit</option>
                        <option value="checkout">Checkout</option>
                        <option value="clone">Clone</option>
                      </select>
                    </label>
                    {#if nodeConfig.operation === "checkout" || nodeConfig.operation === "clone"}
                      <label class="block">
                        <span class="text-xs text-text-muted font-medium">{nodeConfig.operation === "clone" ? "Clone URL" : "Branch"}</span>
                        <input
                          type="text"
                          class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
                          placeholder={nodeConfig.operation === "clone" ? "https://github.com/..." : "main"}
                          value={nodeConfig.branch ?? ""}
                          oninput={(e) => updateNodeConfig(selectedNode!.id, "branch", (e.target as HTMLInputElement).value)}
                        />
                      </label>
                    {/if}
                    {#if nodeConfig.operation === "commit"}
                      <label class="block">
                        <span class="text-xs text-text-muted font-medium">Commit Message</span>
                        <input
                          type="text"
                          class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                          placeholder="feat: update pipeline"
                          value={nodeConfig.message ?? ""}
                          oninput={(e) => updateNodeConfig(selectedNode!.id, "message", (e.target as HTMLInputElement).value)}
                        />
                      </label>
                    {/if}
                  </div>

                <!-- Filter (If) -->
                {:else if nodeType === "filter"}
                  <div class="space-y-3">
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">Condition</span>
                      <select
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                        value={nodeConfig.condition ?? "not_empty"}
                        onchange={(e) => updateNodeConfig(selectedNode!.id, "condition", (e.target as HTMLSelectElement).value)}
                      >
                        <option value="not_empty">Is not empty</option>
                        <option value="empty">Is empty</option>
                        <option value="contains">Contains</option>
                        <option value="not_contains">Does not contain</option>
                        <option value="equals">Equals</option>
                        <option value="not_equals">Does not equal</option>
                        <option value="regex">Matches regex</option>
                      </select>
                    </label>
                    {#if !["not_empty", "empty"].includes(nodeConfig.condition ?? "not_empty")}
                      <label class="block">
                        <span class="text-xs text-text-muted font-medium">Value</span>
                        <input
                          type="text"
                          class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
                          placeholder={nodeConfig.condition === "regex" ? "^\\d+$" : "search text"}
                          value={nodeConfig.value ?? ""}
                          oninput={(e) => updateNodeConfig(selectedNode!.id, "value", (e.target as HTMLInputElement).value)}
                        />
                      </label>
                    {/if}
                    <p class="text-[10px] text-text-muted">Passes input through if condition is met. Errors if not.</p>
                  </div>

                <!-- Read File -->
                {:else if nodeType === "readfile"}
                  <div class="space-y-2">
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">File Path</span>
                      <input
                        type="text"
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
                        placeholder="/path/to/file.txt"
                        value={nodeConfig.path ?? ""}
                        oninput={(e) => updateNodeConfig(selectedNode!.id, "path", (e.target as HTMLInputElement).value)}
                      />
                    </label>
                    <p class="text-[10px] text-text-muted">Reads file contents as output. Supports {"{{"}input{"}}"} in path.</p>
                  </div>

                <!-- Write File -->
                {:else if nodeType === "writefile"}
                  <div class="space-y-3">
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">File Path</span>
                      <input
                        type="text"
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
                        placeholder="/path/to/output.txt"
                        value={nodeConfig.path ?? ""}
                        oninput={(e) => updateNodeConfig(selectedNode!.id, "path", (e.target as HTMLInputElement).value)}
                      />
                    </label>
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">Mode</span>
                      <select
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                        value={nodeConfig.mode ?? "overwrite"}
                        onchange={(e) => updateNodeConfig(selectedNode!.id, "mode", (e.target as HTMLSelectElement).value)}
                      >
                        <option value="overwrite">Overwrite</option>
                        <option value="append">Append</option>
                      </select>
                    </label>
                    <p class="text-[10px] text-text-muted">Writes previous output to file. Supports {"{{"}input{"}}"} in path.</p>
                  </div>

                <!-- Notification -->
                {:else if nodeType === "notification"}
                  <div class="space-y-3">
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">Title</span>
                      <input
                        type="text"
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                        placeholder="Pipeline Complete"
                        value={nodeConfig.title ?? "Pipeline"}
                        oninput={(e) => updateNodeConfig(selectedNode!.id, "title", (e.target as HTMLInputElement).value)}
                      />
                    </label>
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">Body</span>
                      <input
                        type="text"
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                        placeholder="All tasks finished successfully"
                        value={nodeConfig.body ?? ""}
                        oninput={(e) => updateNodeConfig(selectedNode!.id, "body", (e.target as HTMLInputElement).value)}
                      />
                    </label>
                    <p class="text-[10px] text-text-muted">Sends a macOS notification. Supports {"{{"}input{"}}"} and {"{{"}NodeName{"}}"} in both fields.</p>
                  </div>

                <!-- JSON Extract -->
                {:else if nodeType === "jsonextract"}
                  <div class="space-y-3">
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">JSON Path</span>
                      <input
                        type="text"
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
                        placeholder="data.items[0].name"
                        value={nodeConfig.path ?? ""}
                        oninput={(e) => updateNodeConfig(selectedNode!.id, "path", (e.target as HTMLInputElement).value)}
                      />
                    </label>
                    <label class="block">
                      <span class="text-xs text-text-muted font-medium">Fallback Value</span>
                      <input
                        type="text"
                        class="w-full mt-1 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
                        placeholder="(empty if not found)"
                        value={nodeConfig.fallback ?? ""}
                        oninput={(e) => updateNodeConfig(selectedNode!.id, "fallback", (e.target as HTMLInputElement).value)}
                      />
                    </label>
                    <p class="text-[10px] text-text-muted">Extract a value from JSON. Use dot notation: <code class="text-accent">data.items[0].name</code></p>
                  </div>
                {/if}

                <!-- Test result -->
                {#if testResult}
                  <div class="border-t border-border pt-3 mt-2">
                    <div class="flex items-center justify-between mb-1">
                      <span class="text-[10px] font-medium {testResult.error ? 'text-danger' : 'text-success'}">
                        {testResult.error ? "Error" : "Test Output"}
                      </span>
                      <button class="text-[10px] text-text-muted hover:text-text-primary" onclick={() => (testResult = null)}>Clear</button>
                    </div>
                    <pre class="text-[10px] text-text-secondary font-mono whitespace-pre-wrap max-h-40 overflow-y-auto bg-bg-tertiary rounded p-2">{testResult.output.slice(0, 1000)}{testResult.output.length > 1000 ? "..." : ""}</pre>
                  </div>
                {/if}
              </div>

            <!-- Results Tab -->
            {:else if rightTab === "results"}
              <div class="flex-1 overflow-y-auto flex flex-col">
                {#if interactiveNodeId}
                  <!-- Interactive terminal for Claude node -->
                  <div class="border-b border-border px-3 py-2 flex items-center justify-between shrink-0">
                    <div class="flex items-center gap-2">
                      <span class="animate-pulse w-2 h-2 rounded-full bg-warning"></span>
                      <span class="text-xs font-medium text-warning">Interactive — {interactiveNodeLabel}</span>
                    </div>
                    <span class="text-[10px] text-text-muted">Type to interact with Claude</span>
                  </div>
                  <div
                    class="flex-1 min-h-[300px] bg-[#0d1117]"
                    style="padding: 8px;"
                    use:attachTerminalAction
                  ></div>
                {:else if results.length === 0}
                  <div class="flex flex-col items-center justify-center h-full text-text-muted p-4">
                    <Play size={20} class="opacity-30 mb-2" />
                    <p class="text-xs">Run the pipeline to see results</p>
                  </div>
                {:else}
                  <div class="flex items-center justify-between px-3 py-2 border-b border-border">
                    <span class="text-[10px] text-text-muted">{results.filter(r => r.nodeId).length} steps</span>
                    <button class="text-text-muted hover:text-text-primary text-[10px]" onclick={() => clearResults()}>Clear</button>
                  </div>
                  {#each results as result}
                    {#if !result.nodeId}
                      <!-- Pipeline-level event (started/completed) -->
                      <div class="px-3 py-1.5 text-[10px] text-text-muted border-b border-border/30">{result.output}</div>
                    {:else}
                      <!-- Node trace entry -->
                      <details
                        class="border-b border-border/50 group {highlightedResultId === result.nodeId ? 'bg-accent/10' : ''}"
                        id="trace-{result.nodeId}"
                        open={highlightedResultId === result.nodeId}
                      >
                        <summary class="px-3 py-2 cursor-pointer hover:bg-bg-hover list-none flex items-center gap-2">
                          <span class="w-2 h-2 rounded-full shrink-0 {
                            result.status === 'done' ? 'bg-success' :
                            result.status === 'error' ? 'bg-danger' :
                            result.status === 'running' ? 'bg-warning animate-pulse' : 'bg-text-muted'
                          }"></span>
                          <span class="text-xs font-medium text-text-primary flex-1 truncate">{result.label}</span>
                          {#if result.duration > 0}
                            <span class="text-[10px] text-text-muted shrink-0">{result.duration.toLocaleString()}ms</span>
                          {/if}
                        </summary>
                        <div class="px-3 pb-3 space-y-2">
                          {#if result.input}
                            <div>
                              <span class="text-[10px] text-text-muted font-medium uppercase tracking-wider">Input</span>
                              <pre class="text-[10px] text-text-secondary font-mono whitespace-pre-wrap max-h-32 overflow-y-auto bg-bg-tertiary rounded p-2 mt-1">{result.input}</pre>
                            </div>
                          {/if}
                          <div>
                            <span class="text-[10px] text-text-muted font-medium uppercase tracking-wider">Output</span>
                            <pre class="text-[10px] font-mono whitespace-pre-wrap max-h-48 overflow-y-auto bg-bg-tertiary rounded p-2 mt-1 {
                              result.status === 'error' ? 'text-danger' : 'text-text-secondary'
                            }">{result.output}</pre>
                          </div>
                          {#if result.config}
                            <div>
                              <span class="text-[10px] text-text-muted font-medium uppercase tracking-wider">Config</span>
                              <pre class="text-[10px] text-text-muted font-mono whitespace-pre-wrap max-h-24 overflow-y-auto bg-bg-tertiary rounded p-2 mt-1">{result.config}</pre>
                            </div>
                          {/if}
                        </div>
                      </details>
                    {/if}
                  {/each}
                {/if}
              </div>

            <!-- Logs Tab -->
            {:else if rightTab === "logs"}
              <div class="flex-1 overflow-y-auto">
                {#if scheduleLogs.length === 0}
                  <div class="flex flex-col items-center justify-center h-full text-text-muted p-4">
                    <CalendarClock size={20} class="opacity-30 mb-2" />
                    <p class="text-xs">No schedule logs yet</p>
                    {#if !scheduleEnabled}
                      <p class="text-[10px] mt-1">Enable a schedule to see execution logs</p>
                    {/if}
                  </div>
                {:else}
                  <div class="flex items-center justify-between px-3 py-2 border-b border-border">
                    <span class="text-[10px] text-text-muted">{scheduleLogs.length} runs</span>
                    <button class="text-text-muted hover:text-text-primary text-[10px]" onclick={loadScheduleLogs}>Refresh</button>
                  </div>
                  {#each scheduleLogs as log}
                    <details class="border-b border-border/50">
                      <summary class="px-3 py-2 text-xs text-text-secondary cursor-pointer hover:bg-bg-hover">
                        <span class="font-mono text-[10px]">{log.timestamp}</span>
                      </summary>
                      <pre class="px-3 py-2 text-[10px] text-text-secondary font-mono whitespace-pre-wrap max-h-64 overflow-y-auto bg-bg-tertiary m-2 rounded p-2">{log.content}</pre>
                    </details>
                  {/each}
                {/if}
              </div>
            {/if}
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
