<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { Braces } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
  const subtitle = $derived(data.config?.path || "Set JSON path...");
</script>

<BaseNode label={data.label ?? "JSON Extract"} {subtitle} status={data.status} color="border-accent">
  {#snippet icon()}<Braces size={14} class="text-accent" />{/snippet}
</BaseNode>
