<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { Filter } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
  const subtitle = $derived(data.config?.condition || "not_empty");
</script>

<BaseNode label={data.label ?? "Filter"} {subtitle} status={data.status} color="border-accent">
  {#snippet icon()}<Filter size={14} class="text-accent" />{/snippet}
</BaseNode>
