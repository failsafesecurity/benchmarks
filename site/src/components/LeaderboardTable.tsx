import { useMemo, useState } from "react";
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
  type SortingState,
} from "@tanstack/react-table";
import type { RunSummary, LeaderboardData } from "../data/types";
import { formatUsd, formatMs, formatPercent } from "../utils/formatting";
import { frameworkColor } from "../utils/colors";
import { Link } from "react-router-dom";

const col = createColumnHelper<RunSummary>();

interface Props {
  runs: RunSummary[];
  data: LeaderboardData;
}

export default function LeaderboardTable({ runs, data }: Props) {
  const [sorting, setSorting] = useState<SortingState>([
    { id: "pass_rate", desc: true },
  ]);

  const frameworkMap = useMemo(() => {
    const m = new Map<string, string>();
    for (const fw of data.frameworks) m.set(fw.id, fw.name);
    return m;
  }, [data.frameworks]);

  const modelMap = useMemo(() => {
    const m = new Map<string, string>();
    for (const model of data.models) m.set(model.id, model.name);
    return m;
  }, [data.models]);

  const columns = useMemo(
    () => [
      col.accessor("framework_id", {
        header: "Framework",
        cell: (info) => {
          const fwId = info.getValue();
          const color = frameworkColor(fwId);
          return (
            <span className="flex items-center gap-1.5">
              <span
                className="w-2 h-2 rounded-full inline-block"
                style={{ backgroundColor: color }}
              />
              {frameworkMap.get(fwId) ?? fwId}
            </span>
          );
        },
      }),
      col.accessor("framework_version", {
        header: "Version",
        cell: (info) => (
          <span className="text-gray-400">{info.getValue() || "-"}</span>
        ),
      }),
      col.accessor("model_id", {
        header: "Model",
        cell: (info) => modelMap.get(info.getValue()) ?? info.getValue(),
      }),
      col.accessor("suite_id", {
        header: "Suite",
      }),
      col.accessor("pass_rate", {
        header: "Pass Rate",
        cell: (info) => (
          <span className="font-mono">{formatPercent(info.getValue())}</span>
        ),
      }),
      col.accessor("avg_score", {
        header: "Avg Score",
        cell: (info) => (
          <span className="font-mono">{info.getValue().toFixed(3)}</span>
        ),
      }),
      col.accessor("total_cost_usd", {
        header: "Cost",
        cell: (info) => (
          <span className="font-mono">{formatUsd(info.getValue())}</span>
        ),
      }),
      col.accessor("total_wall_time_ms", {
        header: "Time",
        cell: (info) => (
          <span className="font-mono">{formatMs(info.getValue())}</span>
        ),
      }),
      col.accessor("value_score", {
        header: "Value",
        cell: (info) => (
          <span className="font-mono">{info.getValue().toFixed(0)}</span>
        ),
      }),
      col.accessor("run_id", {
        header: "",
        enableSorting: false,
        cell: (info) => (
          <Link
            to={`/run/${info.getValue()}`}
            className="text-orange-400 hover:text-orange-300 text-xs"
          >
            Details
          </Link>
        ),
      }),
    ],
    [frameworkMap, modelMap],
  );

  const table = useReactTable({
    data: runs,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          {table.getHeaderGroups().map((hg) => (
            <tr key={hg.id} className="border-b border-gray-800">
              {hg.headers.map((header) => (
                <th
                  key={header.id}
                  onClick={header.column.getToggleSortingHandler()}
                  className={`px-3 py-2 text-left font-medium text-gray-400 ${
                    header.column.getCanSort()
                      ? "cursor-pointer select-none hover:text-gray-200"
                      : ""
                  }`}
                >
                  <span className="flex items-center gap-1">
                    {flexRender(
                      header.column.columnDef.header,
                      header.getContext(),
                    )}
                    {header.column.getIsSorted() === "asc"
                      ? " \u2191"
                      : header.column.getIsSorted() === "desc"
                        ? " \u2193"
                        : ""}
                  </span>
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row) => (
            <tr
              key={row.id}
              className="border-b border-gray-800/50 hover:bg-gray-900/50"
            >
              {row.getVisibleCells().map((cell) => (
                <td key={cell.id} className="px-3 py-2">
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {runs.length === 0 && (
        <p className="text-center text-gray-500 py-8">
          No runs match the current filters.
        </p>
      )}
    </div>
  );
}
