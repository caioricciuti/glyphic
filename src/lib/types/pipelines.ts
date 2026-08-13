import type { Node, NodeProps } from "@xyflow/svelte";

export type PipelineNodeStatus = "idle" | "running" | "done" | "error";

export interface PipelineNodeConfig {
  prompt?: string;
  command?: string;
  url?: string;
  method?: string;
  headers?: string;
  body?: string;
  operation?: string;
  seconds?: string;
  path?: string;
  branch?: string;
  message?: string;
  condition?: string;
  value?: string;
  mode?: string;
  title?: string;
  fallback?: string;
}

export interface PipelineNodeData extends Record<string, unknown> {
  label?: string;
  config?: PipelineNodeConfig;
  status?: PipelineNodeStatus;
}

export type PipelineNode = Node<PipelineNodeData>;
export type PipelineNodeProps = NodeProps<PipelineNode>;
