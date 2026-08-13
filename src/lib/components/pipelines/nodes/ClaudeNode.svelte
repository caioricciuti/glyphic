<script lang="ts">
  import type { PipelineNodeProps } from "$lib/types";
  import BaseNode from "./BaseNode.svelte";
  import { Bot } from "lucide-svelte";

  let { data }: PipelineNodeProps = $props();
</script>

<BaseNode label={data.label ?? "Claude Prompt"} subtitle={data.config?.prompt?.slice(0, 40) || "Set prompt..."} status={data.status} color="border-accent">
  {#snippet icon()}<Bot size={14} class="text-accent" />{/snippet}
</BaseNode>
