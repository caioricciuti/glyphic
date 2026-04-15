export type Page =
  | "dashboard"
  | "settings"
  | "hooks"
  | "instructions"
  | "memory"
  | "mcp"
  | "skills"
  | "rules"
  | "plugins"
  | "git"
  | "terminal"
  | "analytics"
  | "templates"
  | "sessions"
  | "pipelines"
  | "token-savings"
  | "context-engine"
  | "keybindings";

export interface NavItem {
  id: Page;
  label: string;
  icon: string;
}

export const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "chart" },
  { id: "settings", label: "Settings", icon: "gear" },
  { id: "hooks", label: "Hooks", icon: "bolt" },
  { id: "instructions", label: "Instructions", icon: "book" },
  { id: "memory", label: "Memory", icon: "brain" },
  { id: "mcp", label: "MCP Servers", icon: "server" },
  { id: "skills", label: "Skills & Agents", icon: "sparkles" },
  { id: "rules", label: "Rules", icon: "shield" },
  { id: "plugins", label: "Plugins", icon: "puzzle" },
  { id: "git", label: "Git", icon: "git" },
  { id: "pipelines", label: "Pipelines", icon: "pipelines" },
  { id: "sessions", label: "Sessions", icon: "sessions" },
  { id: "templates", label: "Templates", icon: "templates" },
  { id: "terminal", label: "Terminal", icon: "terminal" },
  { id: "analytics", label: "Analytics", icon: "analytics" },
  { id: "token-savings", label: "Token Savings", icon: "savings" },
  { id: "context-engine", label: "Context Engine", icon: "network" },
  { id: "keybindings", label: "Keybindings", icon: "keybindings" },
];

let currentPage = $state<Page>("dashboard");

export function navigateTo(page: Page) {
  currentPage = page;
}

export function getCurrentPage(): Page {
  return currentPage;
}
