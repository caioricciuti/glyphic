<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { Bell } from "lucide-svelte";

  let { data }: NodeProps = $props();
  const subtitle = $derived(((data as any).config)?.title || "Set title...");
</script>

<BaseNode label={((data as any).label) ?? "Notification"} {subtitle} status={((data as any).status)} color="border-warning">
  {#snippet icon()}<Bell size={14} class="text-warning" />{/snippet}
</BaseNode>
