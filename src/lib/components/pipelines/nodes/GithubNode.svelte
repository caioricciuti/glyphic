<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { GitBranch } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
</script>

<BaseNode label={data.label ?? "GitHub"} subtitle={data.config?.command?.slice(0, 40) || "Set gh command..."} status={data.status} color="border-info">
  {#snippet icon()}<GitBranch size={14} class="text-info" />{/snippet}
</BaseNode>
