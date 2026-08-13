<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { Bell } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
  const subtitle = $derived(data.config?.title || "Set title...");
</script>

<BaseNode label={data.label ?? "Notification"} {subtitle} status={data.status} color="border-warning">
  {#snippet icon()}<Bell size={14} class="text-warning" />{/snippet}
</BaseNode>
