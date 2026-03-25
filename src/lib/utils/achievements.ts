import type { Achievement, StatsCache, Settings } from "$lib/types";

const LEVEL_THRESHOLDS = [
  { level: 1, xp: 0, name: "Newcomer" },
  { level: 2, xp: 100, name: "Apprentice" },
  { level: 3, xp: 300, name: "Practitioner" },
  { level: 4, xp: 600, name: "Specialist" },
  { level: 5, xp: 1000, name: "Expert" },
  { level: 6, xp: 1500, name: "Master" },
  { level: 7, xp: 2500, name: "Architect" },
  { level: 8, xp: 4000, name: "Grandmaster" },
  { level: 9, xp: 6000, name: "Legend" },
  { level: 10, xp: 10000, name: "Transcendent" },
] as const;

export function calculateXP(stats: StatsCache | null, settings: Settings | null): {
  level: number;
  levelName: string;
  currentXP: number;
  nextLevelXP: number;
  totalXP: number;
} {
  let xp = 0;

  if (settings && Object.keys(settings).length > 0) xp += 50;
  if (settings?.hooks) xp += Object.keys(settings.hooks).length * 20;
  if (settings?.mcpServers) xp += Object.keys(settings.mcpServers).length * 30;

  if (stats?.dailyActivity) {
    const activeDays = stats.dailyActivity.filter((d) => d.messageCount > 0).length;
    xp += activeDays * 5;
  }

  let level = 1;
  let levelName = "Newcomer";
  let nextLevelXP = 100;

  for (let i = LEVEL_THRESHOLDS.length - 1; i >= 0; i--) {
    if (xp >= LEVEL_THRESHOLDS[i].xp) {
      level = LEVEL_THRESHOLDS[i].level;
      levelName = LEVEL_THRESHOLDS[i].name;
      nextLevelXP = i < LEVEL_THRESHOLDS.length - 1 ? LEVEL_THRESHOLDS[i + 1].xp : LEVEL_THRESHOLDS[i].xp;
      break;
    }
  }

  return { level, levelName, currentXP: xp, nextLevelXP, totalXP: xp };
}

export function evaluateAchievements(
  stats: StatsCache | null,
  settings: Settings | null,
  currentStreak: number,
): Achievement[] {
  const totalToolCalls = stats?.dailyActivity?.reduce((sum, d) => sum + d.toolCallCount, 0) ?? 0;
  const totalMessages = stats?.totalMessages ?? 0;
  const totalSessions = stats?.totalSessions ?? 0;
  const daysActive = stats?.dailyActivity?.filter((d) => d.messageCount > 0).length ?? 0;
  const hookCount = settings?.hooks ? Object.keys(settings.hooks).length : 0;
  const mcpCount = settings?.mcpServers ? Object.keys(settings.mcpServers).length : 0;
  const hasPermissions = !!settings?.permissions;
  const hasEnvVars = settings?.env && Object.keys(settings.env).length > 0;

  return [
    // Config achievements
    {
      id: "first-settings", name: "Configured", description: "Customized global settings",
      icon: "Settings", unlocked: (settings && Object.keys(settings).length > 1) ?? false, category: "config",
    },
    {
      id: "hook-master", name: "Hook Master", description: "Set up 5+ hook events",
      icon: "Zap", unlocked: hookCount >= 5, category: "config",
    },
    {
      id: "mcp-connected", name: "Connected", description: "Added first MCP server",
      icon: "Server", unlocked: mcpCount > 0, category: "config",
    },
    {
      id: "mcp-network", name: "Network Builder", description: "Connected 3+ MCP servers",
      icon: "Network", unlocked: mcpCount >= 3, category: "config",
    },
    {
      id: "permissions-set", name: "Gatekeeper", description: "Configured permissions",
      icon: "Shield", unlocked: hasPermissions ?? false, category: "config",
    },
    {
      id: "env-vars", name: "Environmentalist", description: "Set environment variables",
      icon: "Variable", unlocked: hasEnvVars ?? false, category: "config",
    },
    // Usage achievements
    {
      id: "first-100-msgs", name: "Getting Started", description: "100+ messages sent",
      icon: "MessageSquare", unlocked: totalMessages >= 100, category: "usage",
    },
    {
      id: "1k-messages", name: "Conversationalist", description: "1,000+ messages sent",
      icon: "MessageCircle", unlocked: totalMessages >= 1000, category: "usage",
    },
    {
      id: "10k-messages", name: "Power User", description: "10,000+ messages sent",
      icon: "MessagesSquare", unlocked: totalMessages >= 10000, category: "usage",
    },
    {
      id: "100k-messages", name: "Unstoppable", description: "100,000+ messages sent",
      icon: "Rocket", unlocked: totalMessages >= 100000, category: "usage",
    },
    {
      id: "centurion", name: "Centurion", description: "100+ sessions",
      icon: "Trophy", unlocked: totalSessions >= 100, category: "usage",
    },
    {
      id: "tool-wielder", name: "Tool Wielder", description: "10,000+ tool calls",
      icon: "Wrench", unlocked: totalToolCalls >= 10000, category: "usage",
    },
    // Streak achievements
    {
      id: "streak-3", name: "Hat Trick", description: "3-day usage streak",
      icon: "Flame", unlocked: currentStreak >= 3, category: "streak",
    },
    {
      id: "streak-7", name: "Week Warrior", description: "7-day usage streak",
      icon: "CalendarDays", unlocked: currentStreak >= 7, category: "streak",
    },
    {
      id: "streak-14", name: "Fortnight Force", description: "14-day usage streak",
      icon: "CalendarRange", unlocked: currentStreak >= 14, category: "streak",
    },
    {
      id: "streak-30", name: "Monthly Master", description: "30-day usage streak",
      icon: "CalendarCheck", unlocked: currentStreak >= 30, category: "streak",
    },
    // Mastery
    {
      id: "10-days", name: "Devoted", description: "Active for 10+ days",
      icon: "Star", unlocked: daysActive >= 10, category: "mastery",
    },
    {
      id: "30-days", name: "Dedicated", description: "Active for 30+ days",
      icon: "Award", unlocked: daysActive >= 30, category: "mastery",
    },
    {
      id: "first-hook", name: "Automator", description: "Created first hook",
      icon: "Cog", unlocked: hookCount >= 1, category: "mastery",
    },
  ];
}
