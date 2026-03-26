/**
 * Build-time aggregation script.
 *
 * Walks `results/` and `baselines/` directories, reads each `run.json`
 * (and optionally `tasks.jsonl`), and produces `public/data/leaderboard.json`.
 *
 * Usage: npx tsx scripts/build-data.ts
 */

import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

interface RunJson {
  run_id: string;
  suite_id: string;
  config_label: string;
  model: string;
  commit_hash: string;
  harness?: string;
  framework?: string;
  framework_version?: string;
  harness_version?: string;
  dataset_version?: string;
  pass_rate: number;
  avg_score: number;
  total_tasks: number;
  completed_tasks: number;
  total_cost_usd: number;
  total_wall_time_ms: number;
  started_at: string;
  finished_at: string;
}

interface TaskJson {
  task_id: string;
  suite_id: string;
  score: { value: number; label: string; details?: string };
  trace: {
    wall_time_ms: number;
    llm_calls: number;
    input_tokens: number;
    output_tokens: number;
    estimated_cost_usd: number;
    tool_calls: { name: string; duration_ms: number; success: boolean }[];
    turns: number;
    hit_iteration_limit: boolean;
    hit_timeout: boolean;
  };
  response: string;
  started_at: string;
  finished_at: string;
  config_label: string;
  error?: string;
}

interface FrameworkRegistry {
  frameworks: {
    id: string;
    name: string;
    url: string;
    versions: Record<string, string>;
  }[];
}

// Output types matching site/src/data/types.ts
interface LeaderboardData {
  generated_at: string;
  frameworks: { id: string; name: string; url?: string; versions: string[] }[];
  models: { id: string; provider: string; name: string }[];
  suites: { id: string; name: string; task_count: number; description: string }[];
  runs: RunSummary[];
}

interface RunSummary {
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

interface TaskSummary {
  task_id: string;
  score: number;
  label: string;
  cost_usd: number;
  wall_time_ms: number;
  tokens: number;
  turns: number;
  error?: string;
}

const ROOT = path.resolve(__dirname, "../..");
const OUTPUT = path.resolve(__dirname, "../public/data/leaderboard.json");

function findRunDirs(...searchDirs: string[]): string[] {
  const dirs: string[] = [];
  for (const searchDir of searchDirs) {
    const absDir = path.resolve(ROOT, searchDir);
    if (!fs.existsSync(absDir)) continue;
    walkForRunJson(absDir, dirs);
  }
  return dirs;
}

function walkForRunJson(dir: string, results: string[]): void {
  const runJsonPath = path.join(dir, "run.json");
  if (fs.existsSync(runJsonPath)) {
    results.push(dir);
    return;
  }
  let entries: fs.Dirent[];
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    if (entry.isDirectory()) {
      walkForRunJson(path.join(dir, entry.name), results);
    }
  }
}

function parseModelId(model: string): { id: string; provider: string; name: string } {
  // Model strings like "openai/gpt-5.2" or "claude-sonnet-4-20250514"
  if (model.includes("/")) {
    const [provider, ...rest] = model.split("/");
    const name = rest.join("/");
    return { id: model, provider, name };
  }
  // Guess provider from model name
  let provider = "unknown";
  if (model.startsWith("gpt-") || model.startsWith("o1") || model.startsWith("o3") || model.startsWith("o4")) {
    provider = "openai";
  } else if (model.startsWith("claude-")) {
    provider = "anthropic";
  } else if (model.startsWith("gemini-")) {
    provider = "google";
  }
  return { id: model, provider, name: model };
}

function readTasks(dir: string): TaskSummary[] {
  const tasksPath = path.join(dir, "tasks.jsonl");
  if (!fs.existsSync(tasksPath)) return [];
  const content = fs.readFileSync(tasksPath, "utf-8");
  const tasks: TaskSummary[] = [];
  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    try {
      const t: TaskJson = JSON.parse(trimmed);
      tasks.push({
        task_id: t.task_id,
        score: t.score.value,
        label: t.score.label,
        cost_usd: t.trace.estimated_cost_usd,
        wall_time_ms: t.trace.wall_time_ms,
        tokens: t.trace.input_tokens + t.trace.output_tokens,
        turns: t.trace.turns,
        error: t.error,
      });
    } catch {
      // Skip malformed lines
    }
  }
  return tasks;
}

function main() {
  // Load framework registry
  const registryPath = path.resolve(__dirname, "frameworks.json");
  const registry: FrameworkRegistry = JSON.parse(
    fs.readFileSync(registryPath, "utf-8"),
  );

  const runDirs = findRunDirs("results", "baselines");
  console.log(`Found ${runDirs.length} run(s)`);

  const frameworksMap = new Map<string, { name: string; url: string; versions: Set<string> }>();
  const modelsMap = new Map<string, { provider: string; name: string }>();
  const suitesMap = new Map<string, { task_count: number }>();
  const runs: RunSummary[] = [];

  // Seed frameworks from registry
  for (const fw of registry.frameworks) {
    frameworksMap.set(fw.id, {
      name: fw.name,
      url: fw.url,
      versions: new Set(),
    });
  }

  for (const dir of runDirs) {
    const runJsonPath = path.join(dir, "run.json");
    let runJson: RunJson;
    try {
      runJson = JSON.parse(fs.readFileSync(runJsonPath, "utf-8"));
    } catch (e) {
      console.warn(`Skipping ${runJsonPath}: ${e}`);
      continue;
    }

    const frameworkId = runJson.framework || runJson.harness || "ironclaw";
    const frameworkVersion = runJson.framework_version || "";

    // Track framework
    if (!frameworksMap.has(frameworkId)) {
      frameworksMap.set(frameworkId, {
        name: frameworkId,
        url: "",
        versions: new Set(),
      });
    }
    if (frameworkVersion) {
      frameworksMap.get(frameworkId)!.versions.add(frameworkVersion);
    }

    // Track model
    const modelInfo = parseModelId(runJson.model);
    if (!modelsMap.has(modelInfo.id)) {
      modelsMap.set(modelInfo.id, {
        provider: modelInfo.provider,
        name: modelInfo.name,
      });
    }

    // Track suite
    if (!suitesMap.has(runJson.suite_id)) {
      suitesMap.set(runJson.suite_id, { task_count: runJson.total_tasks });
    }

    // Read task-level detail
    const tasks = readTasks(dir);

    // Determine if this is an "official" run (from baselines/)
    const relDir = path.relative(ROOT, dir);
    const isOfficial = relDir.startsWith("baselines");

    const passRate = runJson.pass_rate;
    const costUsd = runJson.total_cost_usd;
    const valueScore = (passRate * 1000) / Math.max(costUsd, 0.01);

    runs.push({
      run_id: runJson.run_id,
      framework_id: frameworkId,
      framework_version: frameworkVersion,
      model_id: modelInfo.id,
      suite_id: runJson.suite_id,
      pass_rate: passRate,
      avg_score: runJson.avg_score,
      total_cost_usd: costUsd,
      total_wall_time_ms: runJson.total_wall_time_ms,
      value_score: valueScore,
      total_tasks: runJson.total_tasks,
      completed_tasks: runJson.completed_tasks,
      started_at: runJson.started_at,
      finished_at: runJson.finished_at,
      is_official: isOfficial,
      tasks: tasks.length > 0 ? tasks : undefined,
    });
  }

  const output: LeaderboardData = {
    generated_at: new Date().toISOString(),
    frameworks: [...frameworksMap.entries()].map(([id, info]) => ({
      id,
      name: info.name,
      url: info.url || undefined,
      versions: [...info.versions].sort(),
    })),
    models: [...modelsMap.entries()].map(([id, info]) => ({
      id,
      provider: info.provider,
      name: info.name,
    })),
    suites: [...suitesMap.entries()].map(([id, info]) => ({
      id,
      name: id,
      task_count: info.task_count,
      description: "",
    })),
    runs,
  };

  // Ensure output directory exists
  fs.mkdirSync(path.dirname(OUTPUT), { recursive: true });
  fs.writeFileSync(OUTPUT, JSON.stringify(output, null, 2));
  console.log(`Wrote ${OUTPUT} (${runs.length} runs, ${modelsMap.size} models, ${frameworksMap.size} frameworks)`);
}

main();
