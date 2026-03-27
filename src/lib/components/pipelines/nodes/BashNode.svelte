<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import { Terminal } from "lucide-svelte";

  let { data }: NodeProps = $props();
</script>

<BaseNode label={((data as any).label) ?? "Bash"} subtitle={((data as any).config)?.command?.slice(0, 40) || "Set command..."} status={((data as any).status)} color="border-success">
  {#snippet icon()}<Terminal size={14} class="text-success" />{/snippet}
</BaseNode>
