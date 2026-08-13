<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { Clock } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
</script>

<BaseNode label={data.label ?? "Delay"} subtitle={`${data.config?.seconds || 1}s`} status={data.status} color="border-text-muted">
  {#snippet icon()}<Clock size={14} class="text-text-muted" />{/snippet}
</BaseNode>
