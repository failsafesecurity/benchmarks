export interface LeaderboardData {
  generated_at: string;
  frameworks: Framework[];
  models: Model[];
  suites: Suite[];
  runs: RunSummary[];
}

export interface Framework {
  id: string;
  name: string;
  url?: string;
  versions: string[];
}

export interface Model {
  id: string;
  provider: string;
  name: string;
}

export interface Suite {
  id: string;
  name: string;
  task_count: number;
  description: string;
}

export interface RunSummary {
  run_id: string;
  framework_id: string;
  framework_version: string;
  model_id: string;
  suite_id: string;
  pass_rate: number;
  avg_score: number;
  total_cost_usd: number;
  total_wall_time_ms: number;
  value_score: number;
  total_tasks: number;
  completed_tasks: number;
  started_at: string;
  finished_at: string;
  is_official: boolean;
  tasks?: TaskSummary[];
}

export interface TaskSummary {
  task_id: string;
  score: number;
  label: string;
  cost_usd: number;
  wall_time_ms: number;
  tokens: number;
  turns: number;
  error?: string;
}

export type MetricTab =
  | "success_rate"
  | "speed"
  | "cost"
  | "value"
  | "graphs";

export type ViewMode = "card" | "table";
