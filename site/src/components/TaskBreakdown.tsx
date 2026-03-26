import { useState } from "react";
import type { TaskSummary } from "../data/types";
import { formatUsd, formatMs } from "../utils/formatting";

interface Props {
  tasks: TaskSummary[];
}

export default function TaskBreakdown({ tasks }: Props) {
  const [expanded, setExpanded] = useState(false);

  if (tasks.length === 0) return null;

  return (
    <div>
      <button
        onClick={() => setExpanded(!expanded)}
        className="text-xs text-gray-400 hover:text-gray-200 flex items-center gap-1"
      >
        {expanded ? "\u25BC" : "\u25B6"} {tasks.length} tasks
      </button>
      {expanded && (
        <div className="mt-2 overflow-x-auto">
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-gray-800 text-gray-500">
                <th className="px-2 py-1 text-left">Task</th>
                <th className="px-2 py-1 text-right">Score</th>
                <th className="px-2 py-1 text-right">Label</th>
                <th className="px-2 py-1 text-right">Cost</th>
                <th className="px-2 py-1 text-right">Time</th>
                <th className="px-2 py-1 text-right">Tokens</th>
              </tr>
            </thead>
            <tbody>
              {tasks.map((t) => (
                <tr
                  key={t.task_id}
                  className="border-b border-gray-800/30 hover:bg-gray-900/30"
                >
                  <td className="px-2 py-1 font-mono">{t.task_id}</td>
                  <td className="px-2 py-1 text-right font-mono">
                    {t.score.toFixed(3)}
                  </td>
                  <td className="px-2 py-1 text-right">
                    <span
                      className={
                        t.label === "pass"
                          ? "text-green-400"
                          : t.label === "fail"
                            ? "text-red-400"
                            : "text-yellow-400"
                      }
                    >
                      {t.label}
                    </span>
                  </td>
                  <td className="px-2 py-1 text-right font-mono">
                    {formatUsd(t.cost_usd)}
                  </td>
                  <td className="px-2 py-1 text-right font-mono">
                    {formatMs(t.wall_time_ms)}
                  </td>
                  <td className="px-2 py-1 text-right font-mono">
                    {t.tokens}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
