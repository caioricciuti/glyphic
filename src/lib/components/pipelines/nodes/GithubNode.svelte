<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { GitBranch } from "lucide-svelte";

  let { data }: NodeProps = $props();
</script>

<BaseNode label={((data as any).label) ?? "GitHub"} subtitle={((data as any).config)?.command?.slice(0, 40) || "Set gh command..."} status={((data as any).status)} color="border-info">
  {#snippet icon()}<GitBranch size={14} class="text-info" />{/snippet}
</BaseNode>
