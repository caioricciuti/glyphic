export interface DailyActivity {
  date: string;
  messageCount: number;
  sessionCount: number;
  toolCallCount: number;
}

export interface StatsCache {
  version: number;
  lastComputedDate: string;
  dailyActivity: DailyActivity[];
  totalSessions: number;
  totalMessages: number;
  firstSessionDate: string;
  hourCounts?: Record<string, number>;
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  icon: string;
  unlocked: boolean;
  unlockedAt?: string;
  category: "config" | "usage" | "streak" | "mastery";
}

export interface UserXP {
  level: number;
  levelName: string;
  currentXP: number;
  nextLevelXP: number;
  totalXP: number;
}
