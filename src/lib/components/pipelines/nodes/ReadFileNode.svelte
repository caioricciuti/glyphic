<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { FileInput } from "lucide-svelte";

  let { data }: NodeProps = $props();
  const subtitle = $derived(((data as any).config)?.path?.slice(0, 30) || "Set path...");
</script>

<BaseNode label={((data as any).label) ?? "Read File"} {subtitle} status={((data as any).status)} color="border-info">
  {#snippet icon()}<FileInput size={14} class="text-info" />{/snippet}
</BaseNode>
