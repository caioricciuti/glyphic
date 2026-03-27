<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { Clock } from "lucide-svelte";

  let { data }: NodeProps = $props();
</script>

<BaseNode label={((data as any).label) ?? "Delay"} subtitle={`${((data as any).config)?.seconds || 1}s`} status={((data as any).status)} color="border-text-muted">
  {#snippet icon()}<Clock size={14} class="text-text-muted" />{/snippet}
</BaseNode>
