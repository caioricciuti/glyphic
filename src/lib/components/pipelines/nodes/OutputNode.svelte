<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { CircleDot } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
</script>

<BaseNode label={data.label ?? "End"} subtitle="output" status={data.status} color="border-text-muted" hasOutput={false}>
  {#snippet icon()}<CircleDot size={14} class="text-text-muted" />{/snippet}
</BaseNode>
