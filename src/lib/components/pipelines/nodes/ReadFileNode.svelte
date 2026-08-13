<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { FileInput } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
  const subtitle = $derived(data.config?.path?.slice(0, 30) || "Set path...");
</script>

<BaseNode label={data.label ?? "Read File"} {subtitle} status={data.status} color="border-info">
  {#snippet icon()}<FileInput size={14} class="text-info" />{/snippet}
</BaseNode>
