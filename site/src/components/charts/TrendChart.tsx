import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { RunSummary, LeaderboardData } from "../../data/types";
import { frameworkColor } from "../../utils/colors";

interface Props {
  runs: RunSummary[];
  data: LeaderboardData;
}

export default function TrendChart({ runs, data }: Props) {
  const frameworkIds = [...new Set(runs.map((r) => r.framework_id))];

  // Group by date, show pass_rate per framework
  const dateMap = new Map<string, Record<string, number>>();
  for (const run of runs) {
    const date = run.started_at.slice(0, 10);
    if (!dateMap.has(date)) dateMap.set(date, {});
    const entry = dateMap.get(date)!;
    // Keep the best pass_rate per framework per date
    if (!entry[run.framework_id] || run.pass_rate > entry[run.framework_id]) {
      entry[run.framework_id] = run.pass_rate;
    }
  }

  const chartData = [...dateMap.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([date, vals]) => ({ date, ...vals }));

  if (chartData.length < 2) {
    return (
      <p className="text-gray-500 text-sm text-center py-8">
        Need at least 2 data points for trend chart.
      </p>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={300}>
      <LineChart data={chartData} margin={{ top: 5, right: 20, bottom: 5, left: 0 }}>
        <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
        <XAxis dataKey="date" tick={{ fill: "#9ca3af", fontSize: 12 }} />
        <YAxis domain={[0, 1]} tick={{ fill: "#9ca3af", fontSize: 12 }} />
        <Tooltip
          contentStyle={{
            backgroundColor: "#1f2937",
            border: "1px solid #374151",
            borderRadius: "0.5rem",
          }}
          labelStyle={{ color: "#f3f4f6" }}
        />
        <Legend />
        {frameworkIds.map((fwId) => {
          const fw = data.frameworks.find((f) => f.id === fwId);
          return (
            <Line
              key={fwId}
              type="monotone"
              dataKey={fwId}
              name={fw?.name ?? fwId}
              stroke={frameworkColor(fwId)}
              strokeWidth={2}
              dot={{ r: 4 }}
              connectNulls
            />
          );
        })}
      </LineChart>
    </ResponsiveContainer>
  );
}
