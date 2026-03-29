export interface OptimizerStatus {
  enabled: boolean;
  sidecarInstalled: boolean;
  sidecarVersion: string | null;
  hookInstalled: boolean;
  savingsLogExists: boolean;
  totalCommandsTracked: number;
}

export interface SavingsSummary {
  totalCommands: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalSaved: number;
  avgSavingsPct: number;
}

export interface SavingsTimeBucket {
  label: string;
  commands: number;
  inputTokens: number;
  outputTokens: number;
  savedTokens: number;
  savingsPct: number;
}

export interface CommandSavings {
  command: string;
  count: number;
  totalSaved: number;
  avgSavingsPct: number;
}

export interface SavingsData {
  summary: SavingsSummary;
  daily: SavingsTimeBucket[];
  topCommands: CommandSavings[];
}

export interface DiscoverOpportunity {
  command: string;
  count: number;
  category: string;
  estimatedSavingsTokens: number;
  hasFilter: boolean;
}

export interface DiscoverResult {
  sessionsScanned: number;
  totalCommands: number;
  opportunities: DiscoverOpportunity[];
  totalPotentialSavings: number;
}

export interface FilterRules {
  path: string;
  rawContent: string;
  filterCount: number;
  builtinCount: number;
}
