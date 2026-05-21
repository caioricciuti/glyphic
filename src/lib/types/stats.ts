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

