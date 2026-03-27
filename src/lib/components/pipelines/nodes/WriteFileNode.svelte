<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { FileOutput } from "lucide-svelte";

  let { data }: NodeProps = $props();
  const subtitle = $derived(`${((data as any).config)?.mode || "overwrite"} → ${((data as any).config)?.path?.slice(0, 20) || "Set path..."}`);
</script>

<BaseNode label={((data as any).label) ?? "Write File"} {subtitle} status={((data as any).status)} color="border-warning">
  {#snippet icon()}<FileOutput size={14} class="text-warning" />{/snippet}
</BaseNode>
