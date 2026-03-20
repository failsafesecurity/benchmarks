import {
  ScatterChart,
  Scatter,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import type { RunSummary, LeaderboardData } from "../../data/types";
import { frameworkColor } from "../../utils/colors";

interface Props {
  runs: RunSummary[];
  data: LeaderboardData;
}

export default function CostVsAccuracyScatter({ runs, data }: Props) {
  const frameworkIds = [...new Set(runs.map((r) => r.framework_id))];

  return (
    <ResponsiveContainer width="100%" height={300}>
      <ScatterChart margin={{ top: 5, right: 20, bottom: 5, left: 0 }}>
        <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
        <XAxis
          type="number"
          dataKey="total_cost_usd"
          name="Cost"
          unit="$"
          tick={{ fill: "#9ca3af", fontSize: 12 }}
        />
        <YAxis
          type="number"
          dataKey="pass_rate"
          name="Pass Rate"
          domain={[0, 1]}
          tick={{ fill: "#9ca3af", fontSize: 12 }}
        />
        <Tooltip
          contentStyle={{
            backgroundColor: "#1f2937",
            border: "1px solid #374151",
            borderRadius: "0.5rem",
          }}
          labelStyle={{ color: "#f3f4f6" }}
          formatter={(value: number, name: string) => {
            if (name === "Pass Rate") return [(value * 100).toFixed(1) + "%", name];
            if (name === "Cost") return ["$" + value.toFixed(4), name];
            return [value, name];
          }}
        />
        <Legend />
        {frameworkIds.map((fwId) => {
          const fw = data.frameworks.find((f) => f.id === fwId);
          const fwRuns = runs.filter((r) => r.framework_id === fwId);
          return (
            <Scatter
              key={fwId}
              name={fw?.name ?? fwId}
              data={fwRuns}
              fill={frameworkColor(fwId)}
            />
          );
        })}
      </ScatterChart>
    </ResponsiveContainer>
  );
}
