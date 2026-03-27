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
  import GitNode from "./nodes/GitNode.svelte";
  import FilterNode from "./nodes/FilterNode.svelte";
  import ReadFileNode from "./nodes/ReadFileNode.svelte";
  import WriteFileNode from "./nodes/WriteFileNode.svelte";
  import NotificationNode from "./nodes/NotificationNode.svelte";
  import JsonExtractNode from "./nodes/JsonExtractNode.svelte";

  import {
    Bot, Terminal, GitBranch, Globe, Repeat, Clock, X,
    GitCommitHorizontal, Filter, FileInput, FileOutput, Bell, Braces,
  } from "lucide-svelte";

  type NodeStatus = "idle" | "running" | "done" | "error";

  interface Props {
    nodes: Node[];
    edges: Edge[];
    nodeStatuses?: Record<string, NodeStatus>;
    onselectnode?: (node: Node | null) => void;
  }

  let {
    nodes = $bindable(),
    edges = $bindable(),
    nodeStatuses = {},
    onselectnode,
  }: Props = $props();

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
    git: GitNode,
    filter: FilterNode,
    readfile: ReadFileNode,
    writefile: WriteFileNode,
    notification: NotificationNode,
    jsonextract: JsonExtractNode,
  };

  // Node palette
  let showPalette = $state(false);
  let palettePos = $state({ x: 0, y: 0 });

  const NODE_PALETTE = [
    { type: "claude", label: "Claude Prompt", icon: Bot, color: "text-accent" },
    { type: "bash", label: "Bash Command", icon: Terminal, color: "text-success" },
    { type: "git", label: "Git", icon: GitCommitHorizontal, color: "text-info" },
    { type: "github", label: "GitHub CLI", icon: GitBranch, color: "text-info" },
    { type: "http", label: "HTTP Request", icon: Globe, color: "text-danger" },
    { type: "filter", label: "Filter (If)", icon: Filter, color: "text-accent" },
    { type: "transform", label: "Transform", icon: Repeat, color: "text-warning" },
    { type: "jsonextract", label: "JSON Extract", icon: Braces, color: "text-accent" },
    { type: "readfile", label: "Read File", icon: FileInput, color: "text-info" },
    { type: "writefile", label: "Write File", icon: FileOutput, color: "text-warning" },
    { type: "notification", label: "Notification", icon: Bell, color: "text-warning" },
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
      type === "http" ? { url: "", method: "GET", headers: "[]", body: "" } :
      type === "transform" ? { operation: "passthrough" } :
      type === "delay" ? { seconds: "1" } :
      type === "git" ? { path: "", operation: "status", branch: "", message: "" } :
      type === "filter" ? { condition: "not_empty", value: "" } :
      type === "readfile" ? { path: "" } :
      type === "writefile" ? { path: "", mode: "overwrite" } :
      type === "notification" ? { title: "Pipeline", body: "" } :
      type === "jsonextract" ? { path: "", fallback: "" } : {};

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

  function onNodeClick(_: MouseEvent, node: Node) {
    onselectnode?.(node);
  }

  function onPaneClick() {
    onselectnode?.(null);
    showPalette = false;
    edgeMenu = null;
  }

  // Edge context menu
  let edgeMenu = $state<{ x: number; y: number; edgeId: string } | null>(null);

  function onEdgeContextMenu(event: MouseEvent, edge: { id: string }) {
    event.preventDefault();
    edgeMenu = { x: event.clientX, y: event.clientY, edgeId: edge.id };
  }

  function deleteEdge(edgeId: string) {
    edges = edges.filter((e) => e.id !== edgeId);
    edgeMenu = null;
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
    onedgecontextmenu={(e: any) => onEdgeContextMenu(e.event ?? e.detail?.event, e.edge ?? e.detail?.edge)}
    onpaneclick={onPaneClick}
    deleteKey={["Delete", "Backspace"]}
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

  <!-- Edge context menu -->
  {#if edgeMenu}
    <button class="fixed inset-0 z-40" onclick={() => (edgeMenu = null)} aria-label="Close"></button>
    <div
      class="fixed bg-bg-secondary border border-border rounded-lg shadow-xl p-1 z-50"
      style="left: {edgeMenu.x}px; top: {edgeMenu.y}px"
    >
      <button
        class="flex items-center gap-2 px-3 py-2 text-xs text-danger hover:bg-bg-hover rounded-md w-full"
        onclick={() => deleteEdge(edgeMenu!.edgeId)}
      >
        <X size={12} />
        Delete connection
      </button>
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
