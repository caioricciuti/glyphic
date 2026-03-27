<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { CircleDot } from "lucide-svelte";

  let { data }: NodeProps = $props();
</script>

<BaseNode label={((data as any).label) ?? "End"} subtitle="output" status={((data as any).status)} color="border-text-muted" hasOutput={false}>
  {#snippet icon()}<CircleDot size={14} class="text-text-muted" />{/snippet}
</BaseNode>
