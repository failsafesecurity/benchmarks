import { useState, useEffect } from "react";
import { useParams, Link } from "react-router-dom";
import type { LeaderboardData, RunSummary } from "../data/types";
import { loadLeaderboardData } from "../data";
import { formatUsd, formatMs, formatPercent } from "../utils/formatting";
import { frameworkColor } from "../utils/colors";
import ScoreGauge from "../components/ScoreGauge";
import TaskBreakdown from "../components/TaskBreakdown";

export default function RunDetailPage() {
  const { runId } = useParams<{ runId: string }>();
  const [data, setData] = useState<LeaderboardData | null>(null);

  useEffect(() => {
    loadLeaderboardData().then(setData);
  }, []);

  if (!data) {
    return <div className="text-gray-400 py-20 text-center">Loading...</div>;
  }

  const run = data.runs.find((r) => r.run_id === runId);
  if (!run) {
    return (
      <div className="text-center py-20">
        <p className="text-gray-400 mb-4">Run not found: {runId}</p>
        <Link to="/" className="text-orange-400 hover:text-orange-300">
          Back to leaderboard
        </Link>
      </div>
    );
  }

  const framework = data.frameworks.find((f) => f.id === run.framework_id);
  const model = data.models.find((m) => m.id === run.model_id);
  const color = frameworkColor(run.framework_id);

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <Link
          to="/"
          className="text-gray-400 hover:text-gray-200 text-sm"
        >
          &larr; Leaderboard
        </Link>
      </div>

      <div className="flex flex-col sm:flex-row gap-6">
        <div className="flex flex-col items-center gap-3">
          <ScoreGauge value={run.pass_rate} size={120} color={color} />
          <div className="text-center">
            <div className="flex items-center gap-2">
              <span
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: color }}
              />
              <span className="font-semibold text-lg">
                {framework?.name ?? run.framework_id}
              </span>
            </div>
            {run.framework_version && (
              <span className="text-sm text-gray-500">
                v{run.framework_version}
              </span>
            )}
          </div>
        </div>

        <div className="flex-1">
          <h1 className="text-2xl font-bold mb-4">Run Detail</h1>
          <dl className="grid grid-cols-2 sm:grid-cols-3 gap-x-6 gap-y-3 text-sm">
            <Detail label="Run ID" value={run.run_id.slice(0, 8)} />
            <Detail label="Model" value={model?.name ?? run.model_id} />
            <Detail label="Suite" value={run.suite_id} />
            <Detail label="Pass Rate" value={formatPercent(run.pass_rate)} />
            <Detail label="Avg Score" value={run.avg_score.toFixed(3)} />
            <Detail label="Cost" value={formatUsd(run.total_cost_usd)} />
            <Detail label="Time" value={formatMs(run.total_wall_time_ms)} />
            <Detail
              label="Tasks"
              value={`${run.completed_tasks}/${run.total_tasks}`}
            />
            <Detail label="Value" value={run.value_score.toFixed(0)} />
            <Detail label="Started" value={new Date(run.started_at).toLocaleString()} />
            <Detail label="Finished" value={new Date(run.finished_at).toLocaleString()} />
            <Detail label="Official" value={run.is_official ? "Yes" : "No"} />
          </dl>
        </div>
      </div>

      {run.tasks && run.tasks.length > 0 && (
        <section>
          <h2 className="text-lg font-semibold mb-3">Task Breakdown</h2>
          <div className="bg-gray-900 rounded-xl border border-gray-800 p-4">
            <TaskBreakdown tasks={run.tasks} />
          </div>
        </section>
      )}
    </div>
  );
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-gray-500">{label}</dt>
      <dd className="font-mono text-gray-200">{value}</dd>
    </div>
  );
}
