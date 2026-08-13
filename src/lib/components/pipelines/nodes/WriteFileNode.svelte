<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { FileOutput } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
  const subtitle = $derived(`${data.config?.mode || "overwrite"} → ${data.config?.path?.slice(0, 20) || "Set path..."}`);
</script>

<BaseNode label={data.label ?? "Write File"} {subtitle} status={data.status} color="border-warning">
  {#snippet icon()}<FileOutput size={14} class="text-warning" />{/snippet}
</BaseNode>
