<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { Globe } from "lucide-svelte";

  let { data }: NodeProps = $props();
  const subtitle = $derived(`${((data as any).config)?.method || "GET"} ${((data as any).config)?.url?.slice(0, 30) || "Set URL..."}`);
</script>

<BaseNode label={((data as any).label) ?? "HTTP Request"} {subtitle} status={((data as any).status)} color="border-danger">
  {#snippet icon()}<Globe size={14} class="text-danger" />{/snippet}
</BaseNode>
