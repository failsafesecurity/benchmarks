import { useState, useEffect, useMemo } from "react";
import type { LeaderboardData, RunSummary } from "../data/types";
import { loadLeaderboardData } from "../data";
import { formatUsd, formatMs, formatPercent } from "../utils/formatting";
import { frameworkColor } from "../utils/colors";

export default function ComparePage() {
  const [data, setData] = useState<LeaderboardData | null>(null);
  const [leftId, setLeftId] = useState<string>("");
  const [rightId, setRightId] = useState<string>("");

  useEffect(() => {
    loadLeaderboardData().then(setData);
  }, []);

  const runMap = useMemo(() => {
    if (!data) return new Map<string, RunSummary>();
    return new Map(data.runs.map((r) => [r.run_id, r]));
  }, [data]);

  const left = runMap.get(leftId);
  const right = runMap.get(rightId);

  if (!data) {
    return <div className="text-gray-400 py-20 text-center">Loading...</div>;
  }

  const frameworkMap = new Map(data.frameworks.map((f) => [f.id, f.name]));
  const modelMap = new Map(data.models.map((m) => [m.id, m.name]));

  function runLabel(run: RunSummary) {
    const fw = frameworkMap.get(run.framework_id) ?? run.framework_id;
    const model = modelMap.get(run.model_id) ?? run.model_id;
    return `${fw} - ${model} (${run.suite_id})`;
  }

  const metrics = left && right
    ? [
        {
          label: "Pass Rate",
          left: formatPercent(left.pass_rate),
          right: formatPercent(right.pass_rate),
          delta: ((right.pass_rate - left.pass_rate) * 100).toFixed(1) + "%",
          better: right.pass_rate >= left.pass_rate,
        },
        {
          label: "Avg Score",
          left: left.avg_score.toFixed(3),
          right: right.avg_score.toFixed(3),
          delta: (right.avg_score - left.avg_score).toFixed(3),
          better: right.avg_score >= left.avg_score,
        },
        {
          label: "Cost",
          left: formatUsd(left.total_cost_usd),
          right: formatUsd(right.total_cost_usd),
          delta: formatUsd(right.total_cost_usd - left.total_cost_usd),
          better: right.total_cost_usd <= left.total_cost_usd,
        },
        {
          label: "Time",
          left: formatMs(left.total_wall_time_ms),
          right: formatMs(right.total_wall_time_ms),
          delta: formatMs(Math.abs(right.total_wall_time_ms - left.total_wall_time_ms)),
          better: right.total_wall_time_ms <= left.total_wall_time_ms,
        },
        {
          label: "Value Score",
          left: left.value_score.toFixed(0),
          right: right.value_score.toFixed(0),
          delta: (right.value_score - left.value_score).toFixed(0),
          better: right.value_score >= left.value_score,
        },
      ]
    : [];

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Head-to-Head Compare</h1>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div>
          <label className="block text-sm text-gray-400 mb-1">Baseline</label>
          <select
            value={leftId}
            onChange={(e) => setLeftId(e.target.value)}
            className="w-full bg-gray-900 border border-gray-700 rounded-md px-3 py-2 text-sm text-gray-200"
          >
            <option value="">Select a run...</option>
            {data.runs.map((r) => (
              <option key={r.run_id} value={r.run_id}>
                {runLabel(r)}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label className="block text-sm text-gray-400 mb-1">Comparison</label>
          <select
            value={rightId}
            onChange={(e) => setRightId(e.target.value)}
            className="w-full bg-gray-900 border border-gray-700 rounded-md px-3 py-2 text-sm text-gray-200"
          >
            <option value="">Select a run...</option>
            {data.runs.map((r) => (
              <option key={r.run_id} value={r.run_id}>
                {runLabel(r)}
              </option>
            ))}
          </select>
        </div>
      </div>

      {left && right && (
        <div className="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800">
                <th className="px-4 py-3 text-left text-gray-400">Metric</th>
                <th className="px-4 py-3 text-right text-gray-400">
                  <span className="flex items-center justify-end gap-1.5">
                    <span
                      className="w-2 h-2 rounded-full"
                      style={{ backgroundColor: frameworkColor(left.framework_id) }}
                    />
                    Baseline
                  </span>
                </th>
                <th className="px-4 py-3 text-right text-gray-400">
                  <span className="flex items-center justify-end gap-1.5">
                    <span
                      className="w-2 h-2 rounded-full"
                      style={{ backgroundColor: frameworkColor(right.framework_id) }}
                    />
                    Comparison
                  </span>
                </th>
                <th className="px-4 py-3 text-right text-gray-400">Delta</th>
              </tr>
            </thead>
            <tbody>
              {metrics.map((m) => (
                <tr key={m.label} className="border-b border-gray-800/50">
                  <td className="px-4 py-2 font-medium">{m.label}</td>
                  <td className="px-4 py-2 text-right font-mono">{m.left}</td>
                  <td className="px-4 py-2 text-right font-mono">{m.right}</td>
                  <td
                    className={`px-4 py-2 text-right font-mono ${
                      m.better ? "text-green-400" : "text-red-400"
                    }`}
                  >
                    {m.delta}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {!left && !right && (
        <p className="text-gray-500 text-center py-12">
          Select two runs to compare.
        </p>
      )}
    </div>
  );
}
