<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { Play } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
</script>

<BaseNode label={data.label ?? "Start"} subtitle="input" status={data.status} color="border-warning" hasInput={false}>
  {#snippet icon()}<Play size={14} class="text-warning" />{/snippet}
</BaseNode>
