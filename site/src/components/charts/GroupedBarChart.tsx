import {
  BarChart,
  Bar,
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
  metric: "pass_rate" | "total_cost_usd" | "total_wall_time_ms" | "value_score";
}

export default function GroupedBarChart({ runs, data, metric }: Props) {
  const modelMap = new Map(data.models.map((m) => [m.id, m.name]));
  const frameworkIds = [...new Set(runs.map((r) => r.framework_id))];
  const modelIds = [...new Set(runs.map((r) => r.model_id))];

  const chartData = modelIds.map((modelId) => {
    const entry: Record<string, string | number> = {
      model: modelMap.get(modelId) ?? modelId,
    };
    for (const fwId of frameworkIds) {
      const run = runs.find(
        (r) => r.model_id === modelId && r.framework_id === fwId,
      );
      entry[fwId] = run ? run[metric] : 0;
    }
    return entry;
  });

  const formatLabel: Record<string, string> = {
    pass_rate: "Pass Rate",
    total_cost_usd: "Cost (USD)",
    total_wall_time_ms: "Time (ms)",
    value_score: "Value Score",
  };

  return (
    <ResponsiveContainer width="100%" height={300}>
      <BarChart data={chartData} margin={{ top: 5, right: 20, bottom: 5, left: 0 }}>
        <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
        <XAxis dataKey="model" tick={{ fill: "#9ca3af", fontSize: 12 }} />
        <YAxis tick={{ fill: "#9ca3af", fontSize: 12 }} />
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
            <Bar
              key={fwId}
              dataKey={fwId}
              name={fw?.name ?? fwId}
              fill={frameworkColor(fwId)}
              radius={[4, 4, 0, 0]}
            />
          );
        })}
      </BarChart>
    </ResponsiveContainer>
  );
}
