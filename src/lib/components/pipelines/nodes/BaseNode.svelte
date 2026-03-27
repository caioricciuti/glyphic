<script lang="ts">
  import { Handle, Position } from "@xyflow/svelte";
  import type { Snippet } from "svelte";

  interface Props {
    label: string;
    subtitle?: string;
    status?: string;
    color: string;
    icon: Snippet;
    hasInput?: boolean;
    hasOutput?: boolean;
    children?: Snippet;
  }

  const { label, subtitle = "", status = "idle", color, icon, hasInput = true, hasOutput = true, children }: Props = $props();

  const statusClasses: Record<string, string> = {
    idle: "",
    running: "!border-warning shadow-warning/20 shadow-lg animate-pulse",
    done: "!border-success shadow-success/20 shadow-md",
    error: "!border-danger shadow-danger/20 shadow-md",
  };
</script>

<div class="bg-bg-secondary border-2 {color} rounded-xl px-4 py-3 min-w-[180px] max-w-[220px] transition-all {statusClasses[status] ?? ''}">
  {#if hasInput}
    <Handle type="target" position={Position.Left} class="!w-3 !h-3 !bg-accent !border-2 !border-bg-primary" />
  {/if}

  <div class="flex items-center gap-2 mb-1">
    <div class="w-7 h-7 rounded-lg bg-bg-tertiary flex items-center justify-center shrink-0">
      {@render icon()}
    </div>
    <div class="min-w-0 flex-1">
      <p class="text-xs font-semibold text-text-primary truncate">{label}</p>
      {#if subtitle}
        <p class="text-[10px] text-text-muted truncate">{subtitle}</p>
      {/if}
    </div>
    {#if status === "running"}
      <div class="w-2 h-2 rounded-full bg-warning animate-pulse shrink-0"></div>
    {:else if status === "done"}
      <div class="w-2 h-2 rounded-full bg-success shrink-0"></div>
    {:else if status === "error"}
      <div class="w-2 h-2 rounded-full bg-danger shrink-0"></div>
    {/if}
  </div>

  {#if children}
    <div class="mt-1">
      {@render children()}
    </div>
  {/if}

  {#if hasOutput}
    <Handle type="source" position={Position.Right} class="!w-3 !h-3 !bg-accent !border-2 !border-bg-primary" />
  {/if}
</div>
