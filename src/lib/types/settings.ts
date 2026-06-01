export type EffortLevel = "low" | "medium" | "high" | "xhigh" | "max";
export type DefaultMode = "default" | "plan" | "acceptEdits" | "dontAsk" | "bypassPermissions";
export type SettingsScope = "global" | "project" | "local" | "mcp-local" | "desktop";

export interface ModelOption {
  value: string;
  label: string;
}

/** Model + effort options sourced live from the installed Claude CLI. */
export interface ClaudeCapabilities {
  version: string | null;
  effortLevels: EffortLevel[];
  modelAliases: ModelOption[];
  modelPinned: ModelOption[];
}

export interface Settings {
  model?: string;
  effortLevel?: EffortLevel;
  alwaysThinkingEnabled?: boolean;
  defaultMode?: DefaultMode;
  autoMemoryEnabled?: boolean;
  additionalDirectories?: string[];
  permissions?: {
    allow?: string[];
    ask?: string[];
    deny?: string[];
  };
  env?: Record<string, string>;
  hooks?: Record<string, HookEventConfig[]>;
  mcpServers?: Record<string, McpServerConfig>;
}

export interface HookEventConfig {
  matcher?: string;
  hooks: HookHandler[];
}

export interface HookHandler {
  type: "command" | "http" | "prompt" | "agent";
  command?: string;
  url?: string;
  headers?: Record<string, string>;
  prompt?: string;
  timeout?: number;
  statusMessage?: string;
  async?: boolean;
  model?: string;
}

export interface McpStdioServer {
  command: string;
  args?: string[];
  env?: Record<string, string>;
}

export interface McpSseServer {
  url: string;
  headers?: Record<string, string>;
}

export type McpServerConfig = McpStdioServer | McpSseServer;
