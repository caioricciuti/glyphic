<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { GitCommitHorizontal } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
  const subtitle = $derived(`${data.config?.operation || "status"} ${data.config?.path?.slice(0, 20) || "Set path..."}`);
</script>

<BaseNode label={data.label ?? "Git"} {subtitle} status={data.status} color="border-info">
  {#snippet icon()}<GitCommitHorizontal size={14} class="text-info" />{/snippet}
</BaseNode>
