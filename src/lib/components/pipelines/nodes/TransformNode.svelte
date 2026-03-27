<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { Repeat } from "lucide-svelte";

  let { data }: NodeProps = $props();
</script>

<BaseNode label={((data as any).label) ?? "Transform"} subtitle={((data as any).config)?.operation || "passthrough"} status={((data as any).status)} color="border-warning">
  {#snippet icon()}<Repeat size={14} class="text-warning" />{/snippet}
</BaseNode>
