<script lang="ts">
  import { SvelteFlow, Controls, Background, MiniMap, BackgroundVariant } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";
  import type { Node, Edge } from "@xyflow/svelte";

  import ClaudeNode from "./nodes/ClaudeNode.svelte";
  import BashNode from "./nodes/BashNode.svelte";
  import GithubNode from "./nodes/GithubNode.svelte";
  import HttpNode from "./nodes/HttpNode.svelte";
  import TransformNode from "./nodes/TransformNode.svelte";
  import DelayNode from "./nodes/DelayNode.svelte";
  import InputNode from "./nodes/InputNode.svelte";
  import OutputNode from "./nodes/OutputNode.svelte";

  import {
    Bot, Terminal, GitBranch, Globe, Repeat, Clock,
    X,
  } from "lucide-svelte";

  type NodeStatus = "idle" | "running" | "done" | "error";

  interface Props {
    nodes: Node[];
    edges: Edge[];
    nodeStatuses?: Record<string, NodeStatus>;
    onnodeschange?: (nodes: Node[]) => void;
    onedgeschange?: (edges: Edge[]) => void;
  }

  let { nodes = $bindable(), edges = $bindable(), nodeStatuses = {} }: Props = $props();

  // Update node statuses in data
  $effect(() => {
    for (const node of nodes) {
      if (nodeStatuses[node.id] && node.data.status !== nodeStatuses[node.id]) {
        node.data = { ...node.data, status: nodeStatuses[node.id] };
      }
    }
  });

  const nodeTypes = {
    claude: ClaudeNode,
    bash: BashNode,
    github: GithubNode,
    http: HttpNode,
    transform: TransformNode,
    delay: DelayNode,
    input: InputNode,
    output: OutputNode,
  };

  // Node palette
  let showPalette = $state(false);
  let palettePos = $state({ x: 0, y: 0 });

  const NODE_PALETTE = [
    { type: "claude", label: "Claude Prompt", icon: Bot, color: "text-accent" },
    { type: "bash", label: "Bash Command", icon: Terminal, color: "text-success" },
    { type: "github", label: "GitHub Action", icon: GitBranch, color: "text-info" },
    { type: "http", label: "HTTP Request", icon: Globe, color: "text-danger" },
    { type: "transform", label: "Transform", icon: Repeat, color: "text-warning" },
    { type: "delay", label: "Delay", icon: Clock, color: "text-text-muted" },
  ];

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    palettePos = { x: e.clientX, y: e.clientY };
    showPalette = true;
  }

  function addNode(type: string, label: string) {
    const id = crypto.randomUUID();
    const config =
      type === "claude" ? { prompt: "" } :
      type === "bash" ? { command: "" } :
      type === "github" ? { command: "" } :
      type === "http" ? { url: "", method: "GET", body: "" } :
      type === "transform" ? { operation: "passthrough" } :
      type === "delay" ? { seconds: "1" } : {};

    nodes = [
      ...nodes,
      {
        id,
        type,
        position: { x: palettePos.x - 300, y: palettePos.y - 100 },
        data: { label, config, status: "idle" },
      },
    ];
    showPalette = false;
  }

  // Node config panel
  let selectedNode = $state<Node | null>(null);

  function onNodeClick(_: MouseEvent, node: Node) {
    selectedNode = node;
  }

  function onPaneClick() {
    selectedNode = null;
    showPalette = false;
  }

  function updateNodeData(nodeId: string, updates: Record<string, unknown>) {
    nodes = nodes.map((n) =>
      n.id === nodeId ? { ...n, data: { ...n.data, ...updates } } : n,
    );
    if (selectedNode?.id === nodeId) {
      selectedNode = nodes.find((n) => n.id === nodeId) ?? null;
    }
  }

  function updateNodeConfig(nodeId: string, key: string, value: string) {
    const node = nodes.find((n) => n.id === nodeId);
    if (!node) return;
    const newConfig = { ...(node.data.config ?? {}), [key]: value };
    updateNodeData(nodeId, { config: newConfig });
  }

  function deleteSelectedNode() {
    if (!selectedNode) return;
    const id = selectedNode.id;
    nodes = nodes.filter((n) => n.id !== id);
    edges = edges.filter((e) => e.source !== id && e.target !== id);
    selectedNode = null;
  }
</script>

<div class="relative w-full h-full" oncontextmenu={onContextMenu}>
  <SvelteFlow
    {nodeTypes}
    bind:nodes
    bind:edges
    fitView
    defaultEdgeOptions={{ animated: true, type: "smoothstep" }}
    onnodeclick={(e: any) => onNodeClick(e.event ?? e.detail?.event, e.node ?? e.detail?.node)}
    onpaneclick={onPaneClick}
    deleteKey="Delete"
    colorMode="dark"
  >
    <Background variant={BackgroundVariant.Dots} gap={20} size={1} />
    <Controls position="bottom-left" />
    <MiniMap position="bottom-right" pannable zoomable />
  </SvelteFlow>

  <!-- Node palette (right-click menu) -->
  {#if showPalette}
    <button class="fixed inset-0 z-40" onclick={() => (showPalette = false)} aria-label="Close"></button>
    <div
      class="fixed bg-bg-secondary border border-border rounded-lg shadow-xl p-2 space-y-0.5 z-50"
      style="left: {palettePos.x}px; top: {palettePos.y}px"
    >
      <p class="text-[10px] text-text-muted px-2 pb-1 uppercase tracking-wider">Add Node</p>
      {#each NODE_PALETTE as nt}
        {@const Icon = nt.icon}
        <button
          class="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-text-secondary hover:bg-bg-hover rounded-md transition-colors"
          onclick={() => addNode(nt.type, nt.label)}
        >
          <Icon size={14} class={nt.color} />
          <span>{nt.label}</span>
        </button>
      {/each}
    </div>
  {/if}

  <!-- Node config panel -->
  {#if selectedNode && selectedNode.type !== "input" && selectedNode.type !== "output"}
    <div
      class="absolute bottom-4 left-1/2 -translate-x-1/2 w-[420px] bg-bg-secondary border border-border rounded-xl shadow-2xl p-4 space-y-3 z-30"
    >
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <input
            type="text"
            class="text-sm font-medium bg-transparent border-b border-transparent hover:border-border focus:border-accent text-text-primary focus:outline-none nodrag"
            value={((selectedNode as any).data.label) ?? ""}
            oninput={(e) => updateNodeData(selectedNode!.id, { label: (e.target as HTMLInputElement).value })}
          />
        </div>
        <div class="flex items-center gap-1">
          <button class="p-1 text-text-muted hover:text-danger rounded" onclick={deleteSelectedNode} aria-label="Delete">
            <X size={14} />
          </button>
        </div>
      </div>

      {#if selectedNode.type === "claude"}
        <label class="block">
          <span class="text-xs text-text-muted">Prompt</span>
          <textarea
            class="nodrag nowheel w-full mt-0.5 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono resize-y focus:outline-none focus:border-accent"
            rows="3"
            placeholder="What should Claude do?"
            value={((selectedNode as any).data.config)?.prompt ?? ""}
            oninput={(e) => updateNodeConfig(selectedNode!.id, "prompt", (e.target as HTMLTextAreaElement).value)}
          ></textarea>
          <p class="text-[9px] text-text-muted mt-0.5">Previous output is automatically passed as context. Use {"{{"}NodeName{"}}"} for specific nodes.</p>
        </label>
      {:else if selectedNode.type === "bash" || selectedNode.type === "github"}
        <label class="block">
          <span class="text-xs text-text-muted">{selectedNode.type === "bash" ? "Command" : "GitHub CLI Command"}</span>
          <input
            type="text"
            class="nodrag w-full mt-0.5 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
            placeholder={selectedNode.type === "bash" ? 'echo "{{input}}"' : "gh pr list"}
            value={((selectedNode as any).data.config)?.command ?? ""}
            oninput={(e) => updateNodeConfig(selectedNode!.id, "command", (e.target as HTMLInputElement).value)}
          />
          <p class="text-[9px] text-text-muted mt-0.5">Use {"{{"}input{"}}"} for previous output or {"{{"}NodeName{"}}"} for specific node</p>
        </label>
      {:else if selectedNode.type === "http"}
        <div class="grid grid-cols-4 gap-2">
          <label class="block">
            <span class="text-xs text-text-muted">Method</span>
            <select
              class="nodrag w-full mt-0.5 px-2 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
              value={((selectedNode as any).data.config)?.method ?? "GET"}
              onchange={(e) => updateNodeConfig(selectedNode!.id, "method", (e.target as HTMLSelectElement).value)}
            >
              <option value="GET">GET</option>
              <option value="POST">POST</option>
              <option value="PUT">PUT</option>
              <option value="DELETE">DELETE</option>
            </select>
          </label>
          <label class="block col-span-3">
            <span class="text-xs text-text-muted">URL</span>
            <input
              type="text"
              class="nodrag w-full mt-0.5 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono focus:outline-none focus:border-accent"
              placeholder="https://api.example.com"
              value={((selectedNode as any).data.config)?.url ?? ""}
              oninput={(e) => updateNodeConfig(selectedNode!.id, "url", (e.target as HTMLInputElement).value)}
            />
          </label>
        </div>
      {:else if selectedNode.type === "transform"}
        <label class="block">
          <span class="text-xs text-text-muted">Operation</span>
          <select
            class="nodrag w-full mt-0.5 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
            value={((selectedNode as any).data.config)?.operation ?? "passthrough"}
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
      {:else if selectedNode.type === "delay"}
        <label class="block">
          <span class="text-xs text-text-muted">Seconds</span>
          <input
            type="number"
            class="nodrag w-full mt-0.5 px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent"
            placeholder="1"
            value={((selectedNode as any).data.config)?.seconds ?? "1"}
            oninput={(e) => updateNodeConfig(selectedNode!.id, "seconds", (e.target as HTMLInputElement).value)}
          />
        </label>
      {/if}
    </div>
  {/if}
</div>

<style>
  :global(.svelte-flow) {
    --xy-background-color: var(--color-bg-primary);
    --xy-node-background-color: transparent;
    --xy-node-border-color: transparent;
    --xy-node-color: var(--color-text-primary);
    --xy-edge-stroke: var(--color-accent);
    --xy-edge-stroke-animated: var(--color-accent);
    --xy-minimap-background: var(--color-bg-secondary);
    --xy-controls-button-background-color: var(--color-bg-secondary);
    --xy-controls-button-border-color: var(--color-border);
    --xy-controls-button-color: var(--color-text-secondary);
    --xy-controls-button-background-color-hover: var(--color-bg-hover);
    --xy-background-pattern-color: var(--color-border);
  }

  :global(.svelte-flow .svelte-flow__node) {
    padding: 0;
    border: none;
    background: transparent;
    box-shadow: none;
  }

  :global(.svelte-flow .svelte-flow__minimap) {
    border: 1px solid var(--color-border);
    border-radius: 8px;
  }

  :global(.svelte-flow .svelte-flow__controls) {
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid var(--color-border);
  }
</style>
