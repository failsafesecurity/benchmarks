import type { RunSummary, Framework } from "../data/types";
import { formatUsd, formatMs } from "../utils/formatting";
import { frameworkColor } from "../utils/colors";
import ScoreGauge from "./ScoreGauge";

interface Props {
  run: RunSummary;
  framework: Framework | undefined;
}

export default function FrameworkCard({ run, framework }: Props) {
  const color = frameworkColor(run.framework_id);
  const name = framework?.name ?? run.framework_id;
  const scoreSum = run.tasks
    ? run.tasks.reduce((acc, t) => acc + t.score, 0).toFixed(2)
    : null;

  return (
    <div className="bg-gray-900 border border-gray-800 rounded-xl p-4 flex flex-col items-center gap-3 hover:border-gray-700 transition-colors">
      <div className="flex items-center gap-2">
        <div
          className="w-2.5 h-2.5 rounded-full"
          style={{ backgroundColor: color }}
        />
        <span className="font-semibold text-sm">{name}</span>
        {run.framework_version && (
          <span className="text-xs text-gray-500">
            v{run.framework_version}
          </span>
        )}
      </div>
      <ScoreGauge value={run.pass_rate} color={color} />
      <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-gray-400 w-full">
        <div>
          Cost: <span className="text-gray-200">{formatUsd(run.total_cost_usd)}</span>
        </div>
        <div>
          Time: <span className="text-gray-200">{formatMs(run.total_wall_time_ms)}</span>
        </div>
        <div title="Sum of partial scores / total tasks">
          Tasks:{" "}
          <span className="text-gray-200">
            {scoreSum !== null ? `${scoreSum}/${run.total_tasks}` : `${run.completed_tasks}/${run.total_tasks}`}
          </span>
        </div>
        <div title="Average score per task (0–1 partial credit)">
          Avg score: <span className="text-gray-200">{run.avg_score.toFixed(3)}</span>
        </div>
      </div>
      <div className="text-xs text-gray-500 w-full text-center" title="Percentage of tasks with score = 1.0 (fully passed)">
        Gauge = % fully passed (score = 1.0)
      </div>
      <div className="text-xs text-gray-600 font-mono truncate w-full text-center" title={run.run_id}>
        {run.run_id}
      </div>
    </div>
  );
}
