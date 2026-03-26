import { invoke } from "@tauri-apps/api/core";
import type {
  Settings,
  SettingsScope,
  StatsCache,
  ProjectInfo,
  MemoryFile,
  InstructionFile,
  SkillInfo,
  RuleFile,
  HookEventConfig,
} from "$lib/types";

export const api = {
  settings: {
    read: (scope: SettingsScope, projectPath?: string) =>
      invoke<Settings>("read_settings", { scope, projectPath }),
    write: (scope: SettingsScope, settings: Settings, projectPath?: string) =>
      invoke<void>("write_settings", { scope, projectPath, settings }),
  },

  stats: {
    get: () => invoke<StatsCache>("get_stats"),
    computeLive: () => invoke<StatsCache>("compute_live_stats"),
  },

  projects: {
    list: () => invoke<ProjectInfo[]>("list_projects"),
  },

  hooks: {
    get: (scope: SettingsScope, projectPath?: string) =>
      invoke<Record<string, HookEventConfig[]>>("get_hooks", { scope, projectPath }),
    set: (scope: SettingsScope, hooks: Record<string, HookEventConfig[]>, projectPath?: string) =>
      invoke<void>("set_hooks", { scope, projectPath, hooks }),
  },

  memory: {
    listFiles: (projectHash: string) =>
      invoke<MemoryFile[]>("list_memory_files", { projectHash }),
    readFile: (projectHash: string, filename: string) =>
      invoke<MemoryFile>("read_memory_file", { projectHash, filename }),
    writeFile: (
      projectHash: string,
      filename: string,
      name: string | null,
      description: string | null,
      memoryType: string | null,
      content: string,
    ) =>
      invoke<void>("write_memory_file", {
        projectHash,
        filename,
        name,
        description,
        memoryType,
        content,
      }),
    deleteFile: (projectHash: string, filename: string) =>
      invoke<void>("delete_memory_file", { projectHash, filename }),
  },

  instructions: {
    read: (scope: string, projectPath?: string) =>
      invoke<InstructionFile>("read_instructions", { scope, projectPath }),
    write: (scope: string, content: string, projectPath?: string) =>
      invoke<void>("write_instructions", { scope, projectPath, content }),
    readReference: (basePath: string, reference: string) =>
      invoke<string>("read_referenced_file", { basePath, reference }),
  },

  mcp: {
    list: (scope: SettingsScope, projectPath?: string) =>
      invoke<Record<string, unknown>>("list_mcp_servers", { scope, projectPath }),
    upsert: (scope: SettingsScope, name: string, config: unknown, projectPath?: string) =>
      invoke<void>("upsert_mcp_server", { scope, projectPath, name, config }),
    delete: (scope: SettingsScope, name: string, projectPath?: string) =>
      invoke<void>("delete_mcp_server", { scope, projectPath, name }),
    getCloudMcps: () => invoke<string[]>("get_cloud_mcps"),
  },

  skills: {
    list: (scope: string, projectPath?: string) =>
      invoke<SkillInfo[]>("list_skills", { scope, projectPath }),
    write: (scope: string, name: string, content: string, projectPath?: string) =>
      invoke<void>("write_skill", { scope, projectPath, name, content }),
    delete: (scope: string, name: string, projectPath?: string) =>
      invoke<void>("delete_skill", { scope, projectPath, name }),
  },

  agents: {
    list: (scope: string, projectPath?: string) =>
      invoke<SkillInfo[]>("list_agents", { scope, projectPath }),
    write: (scope: string, name: string, content: string, projectPath?: string) =>
      invoke<void>("write_agent", { scope, projectPath, name, content }),
    delete: (scope: string, name: string, projectPath?: string) =>
      invoke<void>("delete_agent", { scope, projectPath, name }),
  },

  rules: {
    list: (scope: string, projectPath?: string) =>
      invoke<RuleFile[]>("list_rules", { scope, projectPath }),
    write: (
      scope: string,
      filename: string,
      pathsFilter: string[],
      content: string,
      projectPath?: string,
    ) =>
      invoke<void>("write_rule", { scope, projectPath, filename, pathsFilter, content }),
    delete: (scope: string, filename: string, projectPath?: string) =>
      invoke<void>("delete_rule", { scope, projectPath, filename }),
  },

  plugins: {
    getInstalled: () => invoke<unknown>("get_installed_plugins"),
    getBlocked: () => invoke<unknown>("get_blocked_plugins"),
    getMarketplace: () => invoke<unknown>("get_marketplace_plugins"),
    getInstallCounts: () => invoke<unknown>("get_install_counts"),
    install: (name: string) => invoke<string>("install_plugin", { name }),
  },

  sessions: {
    list: (limit?: number, offset?: number) =>
      invoke<SessionListResult>("list_sessions", { limit, offset }),
    load: (path: string, limit?: number, offset?: number) =>
      invoke<SessionLoadResult>("load_session", { path, limit, offset }),
  },

  git: {
    status: (path: string) =>
      invoke<GitStatus>("git_status", { path }),
    log: (path: string, count?: number) =>
      invoke<GitLogEntry[]>("git_log", { path, count }),
    diff: (path: string) =>
      invoke<string>("git_diff", { path }),
    commit: (path: string, message: string) =>
      invoke<string>("git_commit", { path, message }),
    push: (path: string) =>
      invoke<string>("git_push", { path }),
    pull: (path: string) =>
      invoke<string>("git_pull", { path }),
    branches: (path: string) =>
      invoke<string[]>("git_branches", { path }),
    checkout: (path: string, branch: string) =>
      invoke<string>("git_checkout", { path, branch }),
    init: (path: string) =>
      invoke<string>("git_init", { path }),
    openInTerminal: (path: string) =>
      invoke<void>("open_in_terminal", { path }),
  },
} as const;

export interface GitStatus {
  branch: string;
  is_repo: boolean;
  clean: boolean;
  files: GitFileChange[];
  ahead: number;
  behind: number;
}

export interface GitFileChange {
  status: string;
  path: string;
}

export interface GitLogEntry {
  hash: string;
  message: string;
  author: string;
  date: string;
}

export interface SessionListResult {
  sessions: SessionSummary[];
  total: number;
  has_more: boolean;
}

export interface SessionSummary {
  id: string;
  project_hash: string;
  project_path: string;
  path: string;
  entry_count: number;
  user_messages: number;
  tool_calls: number;
  first_timestamp: string | null;
  last_timestamp: string | null;
  first_message: string | null;
}

export interface SessionLoadResult {
  events: SessionEvent[];
  total: number;
  has_more: boolean;
}

export interface SessionEvent {
  type: string;
  timestamp: string | null;
  content: Record<string, unknown>;
}
