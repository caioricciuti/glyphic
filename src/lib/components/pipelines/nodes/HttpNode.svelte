<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { Globe } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
  const subtitle = $derived(`${data.config?.method || "GET"} ${data.config?.url?.slice(0, 30) || "Set URL..."}`);
</script>

<BaseNode label={data.label ?? "HTTP Request"} {subtitle} status={data.status} color="border-danger">
  {#snippet icon()}<Globe size={14} class="text-danger" />{/snippet}
</BaseNode>
