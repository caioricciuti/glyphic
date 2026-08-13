<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { Terminal } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
</script>

<BaseNode label={data.label ?? "Bash"} subtitle={data.config?.command?.slice(0, 40) || "Set command..."} status={data.status} color="border-success">
  {#snippet icon()}<Terminal size={14} class="text-success" />{/snippet}
</BaseNode>
