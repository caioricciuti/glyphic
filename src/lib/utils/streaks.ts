import type { DailyActivity } from "$lib/types";

export function calculateStreak(dailyActivity: DailyActivity[]): {
  current: number;
  longest: number;
  lastActiveDate: string;
} {
  if (!dailyActivity.length) return { current: 0, longest: 0, lastActiveDate: "" };

  const activeDates = new Set(
    dailyActivity
      .filter((d) => d.messageCount > 0)
      .map((d) => d.date),
  );

  if (activeDates.size === 0) return { current: 0, longest: 0, lastActiveDate: "" };

  const sorted = [...activeDates].sort();
  const lastActiveDate = sorted[sorted.length - 1];

  // Current streak: count backward from most recent active date
  let current = 0;
  let checkDate = new Date(lastActiveDate);
  while (activeDates.has(checkDate.toISOString().split("T")[0])) {
    current++;
    checkDate = new Date(checkDate.getTime() - 86400000);
  }

  // Longest streak
  let longest = 0;
  let streak = 0;
  for (const date of sorted) {
    const prev = new Date(new Date(date).getTime() - 86400000)
      .toISOString()
      .split("T")[0];
    if (activeDates.has(prev)) {
      streak++;
    } else {
      streak = 1;
    }
    longest = Math.max(longest, streak);
  }

  return { current, longest, lastActiveDate };
}

export function getDaysActive(dailyActivity: DailyActivity[]): number {
  return dailyActivity.filter((d) => d.messageCount > 0).length;
}
