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
  OptimizerStatus,
  SavingsData,
  DiscoverResult,
  FilterRules,
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

  maintenance: {
    getDiskUsage: () => invoke<DiskUsageReport>("get_disk_usage"),
    cleanup: (name: string) => invoke<number>("cleanup_directory", { name }),
  },

  budget: {
    get: () => invoke<BudgetSettings>("get_budget"),
    set: (dailyLimit: number | null, monthlyLimit: number | null, planType?: string) =>
      invoke<void>("set_budget", { dailyLimit, monthlyLimit, planType }),
    getCostSummary: () => invoke<CostSummary>("get_cost_summary"),
  },

  sessions: {
    list: (limit?: number, offset?: number) =>
      invoke<SessionListResult>("list_sessions", { limit, offset }),
    load: (path: string, limit?: number, offset?: number) =>
      invoke<SessionLoadResult>("load_session", { path, limit, offset }),
    search: (query: string, maxResults?: number) =>
      invoke<SearchResult[]>("search_sessions", { query, maxResults }),
    getTags: () => invoke<SessionTags>("get_session_tags"),
    setTag: (sessionId: string, tags: string[], note?: string) =>
      invoke<void>("set_session_tag", { sessionId, tags, note }),
    exportMarkdown: (path: string) =>
      invoke<string>("export_session_markdown", { path }),
    detectLive: () => invoke<LiveSession[]>("detect_live_sessions"),
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

  tokenSavings: {
    status: () => invoke<OptimizerStatus>("get_optimizer_status"),
    enable: () => invoke<void>("enable_optimizer"),
    disable: () => invoke<void>("disable_optimizer"),
    savings: (period?: string, projectPath?: string) =>
      invoke<SavingsData>("get_savings_data", {
        period: period ?? "daily",
        projectPath,
      }),
    discover: (projectPath?: string) =>
      invoke<DiscoverResult>("discover_opportunities", { projectPath }),
    getFilters: () => invoke<FilterRules>("get_filter_rules"),
    saveFilters: (content: string) =>
      invoke<void>("save_filter_rules", { content }),
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

export interface DiskUsageReport {
  total_bytes: number;
  total_display: string;
  entries: DiskUsageEntry[];
}

export interface DiskUsageEntry {
  name: string;
  path: string;
  size_bytes: number;
  size_display: string;
  description: string;
  safe_to_delete: boolean;
}

export interface BudgetSettings {
  daily_limit: number | null;
  monthly_limit: number | null;
  plan_type: string;
}

export interface CostSummary {
  today: number;
  this_month: number;
  last_7_days: number[];
  daily_limit: number | null;
  monthly_limit: number | null;
  daily_exceeded: boolean;
  monthly_exceeded: boolean;
  monthly_projection: number;
  per_project_month: ProjectCost[];
  plan_type: string;
}

export interface ProjectCost {
  project: string;
  cost: number;
  messages: number;
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

export interface SearchResult {
  session_id: string;
  project_path: string;
  path: string;
  snippet: string;
  timestamp: string | null;
  event_type: string;
}

export interface SessionTags {
  tags: Record<string, string[]>;
  notes: Record<string, string>;
}

export interface LiveSession {
  path: string;
  project_path: string;
  modified_secs_ago: number;
}
