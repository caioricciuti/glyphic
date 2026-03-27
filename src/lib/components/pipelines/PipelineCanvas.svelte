<script lang="ts">
  import { Bot, Terminal, GitBranch, Play, CircleDot, X, Trash2, Globe, Repeat, Clock } from "lucide-svelte";

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

  type NodeStatus = "idle" | "running" | "done" | "error";

  interface Props {
    nodes: PipelineNode[];
    connections: PipelineConnection[];
    nodeStatuses?: Record<string, NodeStatus>;
  }

  let { nodes = $bindable(), connections = $bindable(), nodeStatuses = {} }: Props = $props();

  let dragging = $state<{ nodeId: string; offsetX: number; offsetY: number } | null>(null);
  let connecting = $state<{ fromNode: string; mouseX: number; mouseY: number } | null>(null);
  let selectedNode = $state<string | null>(null);
  let canvasEl: HTMLDivElement;
  let showAddMenu = $state(false);
  let addMenuPos = $state({ x: 300, y: 300 });

  const NODE_W = 200;
  const NODE_H = 80;

  const NODE_TYPES = [
    { type: "claude", label: "Claude Prompt", icon: Bot, color: "border-accent bg-accent/5" },
    { type: "bash", label: "Bash Command", icon: Terminal, color: "border-success bg-success/5" },
    { type: "github", label: "GitHub Action", icon: GitBranch, color: "border-info bg-info/5" },
    { type: "http", label: "HTTP Request", icon: Globe, color: "border-danger bg-danger/5" },
    { type: "transform", label: "Transform", icon: Repeat, color: "border-warning bg-warning/5" },
    { type: "delay", label: "Delay", icon: Clock, color: "border-text-muted bg-bg-tertiary" },
    { type: "input", label: "Input", icon: Play, color: "border-warning bg-warning/5" },
    { type: "output", label: "Output", icon: CircleDot, color: "border-text-muted bg-bg-tertiary" },
  ];

  function getNodeStyle(type: string): string {
    return NODE_TYPES.find((t) => t.type === type)?.color ?? "border-border bg-bg-secondary";
  }

  function getNodeIcon(type: string) {
    return NODE_TYPES.find((t) => t.type === type)?.icon ?? CircleDot;
  }

  // Dragging
  function startDrag(e: MouseEvent, nodeId: string) {
    if ((e.target as HTMLElement).closest("[data-port]")) return;
    const node = nodes.find((n) => n.id === nodeId);
    if (!node) return;
    const rect = canvasEl.getBoundingClientRect();
    dragging = {
      nodeId,
      offsetX: e.clientX - rect.left - node.x,
      offsetY: e.clientY - rect.top - node.y,
    };
    selectedNode = nodeId;
    e.preventDefault();
  }

  function onMouseMove(e: MouseEvent) {
    if (dragging) {
      const rect = canvasEl.getBoundingClientRect();
      const node = nodes.find((n) => n.id === dragging!.nodeId);
      if (node) {
        node.x = Math.max(0, e.clientX - rect.left - dragging!.offsetX);
        node.y = Math.max(0, e.clientY - rect.top - dragging!.offsetY);
        nodes = [...nodes]; // trigger reactivity
      }
    }
    if (connecting) {
      const rect = canvasEl.getBoundingClientRect();
      connecting = { ...connecting, mouseX: e.clientX - rect.left, mouseY: e.clientY - rect.top };
    }
  }

  function onMouseUp() {
    dragging = null;
    if (connecting) connecting = null;
  }

  // Connections
  function startConnect(e: MouseEvent, nodeId: string) {
    e.stopPropagation();
    const rect = canvasEl.getBoundingClientRect();
    connecting = { fromNode: nodeId, mouseX: e.clientX - rect.left, mouseY: e.clientY - rect.top };
  }

  function endConnect(e: MouseEvent, nodeId: string) {
    e.stopPropagation();
    if (connecting && connecting.fromNode !== nodeId) {
      const exists = connections.some(
        (c) => c.from_node === connecting!.fromNode && c.to_node === nodeId,
      );
      if (!exists) {
        connections = [
          ...connections,
          { id: crypto.randomUUID(), from_node: connecting.fromNode, to_node: nodeId },
        ];
      }
    }
    connecting = null;
  }

  // Add node
  function addNode(type: string) {
    nodes = [
      ...nodes,
      {
        id: crypto.randomUUID(),
        type,
        label: NODE_TYPES.find((t) => t.type === type)?.label ?? type,
        x: addMenuPos.x,
        y: addMenuPos.y,
        config: type === "claude" ? { prompt: "" } :
               type === "bash" ? { command: "" } :
               type === "github" ? { command: "" } :
               type === "http" ? { url: "", method: "GET", body: "" } :
               type === "transform" ? { operation: "passthrough" } :
               type === "delay" ? { seconds: "1" } : {},
      },
    ];
    showAddMenu = false;
  }

  function deleteNode(nodeId: string) {
    nodes = nodes.filter((n) => n.id !== nodeId);
    connections = connections.filter((c) => c.from_node !== nodeId && c.to_node !== nodeId);
    selectedNode = null;
  }

  function deleteConnection(connId: string) {
    connections = connections.filter((c) => c.id !== connId);
  }

  // Connection path
  function getPath(from: PipelineNode, to: PipelineNode): string {
    const x1 = from.x + NODE_W;
    const y1 = from.y + NODE_H / 2;
    const x2 = to.x;
    const y2 = to.y + NODE_H / 2;
    const dx = Math.abs(x2 - x1) * 0.5;
    return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
  }

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    const rect = canvasEl.getBoundingClientRect();
    addMenuPos = { x: e.clientX - rect.left, y: e.clientY - rect.top };
    showAddMenu = true;
    selectedNode = null;
  }
</script>

<div
  bind:this={canvasEl}
  class="relative w-full h-full overflow-hidden bg-bg-primary cursor-crosshair"
  role="application"
  tabindex="-1"
  onmousemove={onMouseMove}
  onmouseup={onMouseUp}
  oncontextmenu={onContextMenu}
  onclick={() => { selectedNode = null; showAddMenu = false; }}
>
  <!-- Grid pattern -->
  <svg class="absolute inset-0 w-full h-full pointer-events-none opacity-10">
    <defs>
      <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
        <path d="M 20 0 L 0 0 0 20" fill="none" stroke="currentColor" stroke-width="0.5" class="text-text-muted" />
      </pattern>
    </defs>
    <rect width="100%" height="100%" fill="url(#grid)" />
  </svg>

  <!-- SVG connections layer -->
  <svg class="absolute inset-0 w-full h-full pointer-events-none" style="z-index: 1">
    {#each connections as conn}
      {@const from = nodes.find((n) => n.id === conn.from_node)}
      {@const to = nodes.find((n) => n.id === conn.to_node)}
      {#if from && to}
        <path
          d={getPath(from, to)}
          fill="none"
          stroke="var(--color-accent)"
          stroke-width="2"
          stroke-opacity="0.6"
          class="pointer-events-auto cursor-pointer hover:stroke-[3]"
          onclick={(e) => { e.stopPropagation(); deleteConnection(conn.id); }}
        />
        <!-- Arrow -->
        {@const mx = (from.x + NODE_W + to.x) / 2}
        {@const my = (from.y + NODE_H / 2 + to.y + NODE_H / 2) / 2}
        <circle cx={mx} cy={my} r="3" fill="var(--color-accent)" opacity="0.6" />
      {/if}
    {/each}

    <!-- Active connection line -->
    {#if connecting}
      {@const conn = connecting}
      {@const from = nodes.find((n) => n.id === conn.fromNode)}
      {#if from}
        <line
          x1={from.x + NODE_W}
          y1={from.y + NODE_H / 2}
          x2={conn.mouseX}
          y2={conn.mouseY}
          stroke="var(--color-accent)"
          stroke-width="2"
          stroke-dasharray="6 4"
          opacity="0.8"
        />
      {/if}
    {/if}
  </svg>

  <!-- Nodes -->
  {#each nodes as node (node.id)}
    {@const Icon = getNodeIcon(node.type)}
    {@const status = nodeStatuses[node.id] ?? "idle"}
    <div
      class="absolute select-none border-2 rounded-xl shadow-lg transition-all
        {status === 'running' ? 'border-warning bg-warning/5 animate-pulse shadow-warning/20' :
         status === 'done' ? 'border-success bg-success/5 shadow-success/20' :
         status === 'error' ? 'border-danger bg-danger/5 shadow-danger/20' :
         getNodeStyle(node.type)}
        {selectedNode === node.id ? 'ring-2 ring-accent shadow-accent/20' : ''}"
      style="left: {node.x}px; top: {node.y}px; width: {NODE_W}px; height: {NODE_H}px; z-index: {selectedNode === node.id ? 10 : 2}"
      role="button"
      tabindex="0"
      onmousedown={(e) => startDrag(e, node.id)}
      onclick={(e) => { e.stopPropagation(); selectedNode = node.id; showAddMenu = false; }}
      onkeydown={(e) => e.key === "Delete" && deleteNode(node.id)}
    >
      <!-- Node content -->
      <div class="flex items-center gap-2 px-3 h-full">
        <div class="w-8 h-8 rounded-lg bg-bg-secondary flex items-center justify-center shrink-0">
          <Icon size={16} class="text-text-secondary" />
        </div>
        <div class="flex-1 min-w-0">
          <p class="text-xs font-medium text-text-primary truncate">{node.label}</p>
          <p class="text-[10px] text-text-muted truncate">
            {node.type === "claude" ? (node.config.prompt || "Set prompt...") :
             node.type === "bash" ? (node.config.command || "Set command...") :
             node.type === "github" ? (node.config.command || "Set gh command...") :
             node.type === "http" ? `${node.config.method || "GET"} ${node.config.url || "Set URL..."}` :
             node.type === "transform" ? (node.config.operation || "passthrough") :
             node.type === "delay" ? `${node.config.seconds || 1}s delay` :
             node.type}
          </p>
        </div>
        {#if selectedNode === node.id && node.type !== "input" && node.type !== "output"}
          <button class="p-1 text-text-muted hover:text-danger shrink-0" onclick={(e) => { e.stopPropagation(); deleteNode(node.id); }} aria-label="Delete node">
            <Trash2 size={12} />
          </button>
        {/if}
      </div>

      <!-- Input port (left) -->
      {#if node.type !== "input"}
        <div
          data-port="input"
          class="absolute -left-2 top-1/2 -translate-y-1/2 w-4 h-4 rounded-full border-2 border-accent bg-bg-primary cursor-pointer hover:bg-accent hover:scale-125 transition-all"
          role="button"
          tabindex="-1"
          onmouseup={(e) => endConnect(e, node.id)}
        ></div>
      {/if}

      <!-- Output port (right) -->
      {#if node.type !== "output"}
        <div
          data-port="output"
          class="absolute -right-2 top-1/2 -translate-y-1/2 w-4 h-4 rounded-full border-2 border-accent bg-bg-primary cursor-pointer hover:bg-accent hover:scale-125 transition-all"
          role="button"
          tabindex="-1"
          onmousedown={(e) => startConnect(e, node.id)}
        ></div>
      {/if}
    </div>
  {/each}

  <!-- Add node menu -->
  {#if showAddMenu}
    <div
      class="absolute bg-bg-secondary border border-border rounded-lg shadow-xl p-2 space-y-1"
      style="left: {addMenuPos.x}px; top: {addMenuPos.y}px; z-index: 20"
    >
      <p class="text-[10px] text-text-muted px-2 pb-1">Add Node</p>
      {#each NODE_TYPES.filter((t) => t.type !== "input" && t.type !== "output") as nt}
        {@const NIcon = nt.icon}
        <button
          class="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-text-secondary hover:bg-bg-hover rounded-md"
          onclick={(e) => { e.stopPropagation(); addNode(nt.type); }}
        >
          <NIcon size={12} />
          {nt.label}
        </button>
      {/each}
    </div>
  {/if}

  <!-- Node config panel -->
  {#if selectedNode}
    {@const node = nodes.find((n) => n.id === selectedNode)}
    {#if node && node.type !== "input" && node.type !== "output"}
      <div
        class="absolute bottom-4 left-1/2 -translate-x-1/2 w-96 bg-bg-secondary border border-border rounded-lg shadow-xl p-4 space-y-2"
        style="z-index: 30"
        onclick={(e) => e.stopPropagation()}
      >
        <div class="flex items-center justify-between">
          <label class="block">
            <span class="text-xs text-text-muted">Label</span>
            <input type="text" class="w-full mt-0.5 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent" bind:value={node.label} />
          </label>
          <button class="p-1 text-text-muted hover:text-text-primary ml-2" onclick={() => (selectedNode = null)} aria-label="Close">
            <X size={14} />
          </button>
        </div>
        {#if node.type === "claude"}
          <label class="block">
            <span class="text-xs text-text-muted">Prompt</span>
            <textarea class="w-full mt-0.5 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary font-mono resize-y focus:outline-none focus:border-accent" rows="3" placeholder="What should Claude do?" bind:value={node.config.prompt}></textarea>
          </label>
        {:else if node.type === "bash"}
          <label class="block">
            <span class="text-xs text-text-muted">Command</span>
            <input type="text" class="w-full mt-0.5 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary font-mono focus:outline-none focus:border-accent" placeholder={"echo '{{input}}'"} bind:value={node.config.command} />
            <p class="text-[9px] text-text-muted mt-0.5">Use <code class="text-accent">{"{{"}input{"}}"}</code> to reference previous output</p>
          </label>
        {:else if node.type === "github"}
          <label class="block">
            <span class="text-xs text-text-muted">GitHub CLI Command</span>
            <input type="text" class="w-full mt-0.5 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary font-mono focus:outline-none focus:border-accent" placeholder="gh pr list" bind:value={node.config.command} />
            <p class="text-[9px] text-text-muted mt-0.5">Use <code class="text-accent">{"{{"}input{"}}"}</code> to reference previous output</p>
          </label>
        {:else if node.type === "http"}
          <div class="grid grid-cols-3 gap-2">
            <label class="block">
              <span class="text-xs text-text-muted">Method</span>
              <select class="w-full mt-0.5 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent" bind:value={node.config.method}>
                <option value="GET">GET</option>
                <option value="POST">POST</option>
                <option value="PUT">PUT</option>
                <option value="DELETE">DELETE</option>
              </select>
            </label>
            <label class="block col-span-2">
              <span class="text-xs text-text-muted">URL</span>
              <input type="text" class="w-full mt-0.5 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary font-mono focus:outline-none focus:border-accent" placeholder="https://api.example.com/data" bind:value={node.config.url} />
            </label>
          </div>
          <label class="block">
            <span class="text-xs text-text-muted">Body (for POST/PUT)</span>
            <textarea class="w-full mt-0.5 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary font-mono resize-y focus:outline-none focus:border-accent" rows="2" placeholder="JSON body..." bind:value={node.config.body}></textarea>
          </label>
        {:else if node.type === "transform"}
          <label class="block">
            <span class="text-xs text-text-muted">Operation</span>
            <select class="w-full mt-0.5 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent" bind:value={node.config.operation}>
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
        {:else if node.type === "delay"}
          <label class="block">
            <span class="text-xs text-text-muted">Seconds</span>
            <input type="number" class="w-full mt-0.5 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent" placeholder="1" bind:value={node.config.seconds} />
          </label>
        {/if}
      </div>
    {/if}
  {/if}

  <!-- Hint -->
  <div class="absolute bottom-3 right-3 text-[10px] text-text-muted opacity-50" style="z-index: 5">
    Right-click to add node · Drag output port to connect · Use {"{{"}input{"}}"} for chaining
  </div>
</div>
