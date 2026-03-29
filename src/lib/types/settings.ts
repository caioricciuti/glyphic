export type EffortLevel = "low" | "medium" | "high";
export type DefaultMode = "default" | "plan" | "acceptEdits" | "dontAsk" | "bypassPermissions";
export type SettingsScope = "global" | "project" | "local" | "mcp-local" | "desktop";

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
