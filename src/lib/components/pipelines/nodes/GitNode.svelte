<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { GitCommitHorizontal } from "lucide-svelte";

  let { data }: NodeProps = $props();
  const subtitle = $derived(`${((data as any).config)?.operation || "status"} ${((data as any).config)?.path?.slice(0, 20) || "Set path..."}`);
</script>

<BaseNode label={((data as any).label) ?? "Git"} {subtitle} status={((data as any).status)} color="border-info">
  {#snippet icon()}<GitCommitHorizontal size={14} class="text-info" />{/snippet}
</BaseNode>
