<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { Filter } from "lucide-svelte";

  let { data }: NodeProps = $props();
  const subtitle = $derived(((data as any).config)?.condition || "not_empty");
</script>

<BaseNode label={((data as any).label) ?? "Filter"} {subtitle} status={((data as any).status)} color="border-accent">
  {#snippet icon()}<Filter size={14} class="text-accent" />{/snippet}
</BaseNode>
