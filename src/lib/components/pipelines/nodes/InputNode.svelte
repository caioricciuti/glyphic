<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { Play } from "lucide-svelte";

  let { data }: NodeProps = $props();
</script>

<BaseNode label={((data as any).label) ?? "Start"} subtitle="input" status={((data as any).status)} color="border-warning" hasInput={false}>
  {#snippet icon()}<Play size={14} class="text-warning" />{/snippet}
</BaseNode>
