<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { Repeat } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
</script>

<BaseNode label={data.label ?? "Transform"} subtitle={data.config?.operation || "passthrough"} status={data.status} color="border-warning">
  {#snippet icon()}<Repeat size={14} class="text-warning" />{/snippet}
</BaseNode>
