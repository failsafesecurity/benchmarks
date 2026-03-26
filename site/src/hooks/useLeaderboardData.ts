import { useState, useEffect, useMemo } from "react";
import type {
  LeaderboardData,
  RunSummary,
  MetricTab,
  ViewMode,
} from "../data/types";
import { loadLeaderboardData } from "../data";

export interface Filters {
  modelId: string | null;
  suiteId: string | null;
  datasetId: string | null;
  frameworkVersions: Record<string, string>; // framework_id -> version
  latestOnly: boolean;
  officialOnly: boolean;
}

export function useLeaderboardData() {
  const [data, setData] = useState<LeaderboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [metricTab, setMetricTab] = useState<MetricTab>("success_rate");
  const [viewMode, setViewMode] = useState<ViewMode>("card");
  const [filters, setFilters] = useState<Filters>({
    modelId: null,
    suiteId: null,
    datasetId: null,
    frameworkVersions: {},
    latestOnly: true,
    officialOnly: false,
  });

  useEffect(() => {
    loadLeaderboardData()
      .then(setData)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  const filteredRuns = useMemo(() => {
    if (!data) return [];
    let runs = data.runs;

    if (filters.modelId) {
      runs = runs.filter((r) => r.model_id === filters.modelId);
    }
    if (filters.suiteId) {
      runs = runs.filter((r) => r.suite_id === filters.suiteId);
    }
    if (filters.datasetId) {
      runs = runs.filter((r) => r.dataset === filters.datasetId);
    }
    if (filters.officialOnly) {
      runs = runs.filter((r) => r.is_official);
    }
    if (filters.latestOnly) {
      // Keep only the latest run per (framework, model, suite)
      const latest = new Map<string, RunSummary>();
      for (const run of runs) {
        const key = `${run.framework_id}|${run.model_id}|${run.dataset}`;
        const existing = latest.get(key);
        if (!existing || run.started_at > existing.started_at) {
          latest.set(key, run);
        }
      }
      runs = Array.from(latest.values());
    } else {
      // Apply per-framework version filters
      for (const [fwId, version] of Object.entries(
        filters.frameworkVersions,
      )) {
        if (version) {
          runs = runs.filter(
            (r) =>
              r.framework_id !== fwId || r.framework_version === version,
          );
        }
      }
    }

    return runs;
  }, [data, filters]);

  // Group runs by model
  const runsByModel = useMemo(() => {
    const grouped = new Map<string, RunSummary[]>();
    for (const run of filteredRuns) {
      const key = run.model_id;
      if (!grouped.has(key)) grouped.set(key, []);
      grouped.get(key)!.push(run);
    }
    return grouped;
  }, [filteredRuns]);

  const sortedRuns = useMemo(() => {
    const sorted = [...filteredRuns];
    switch (metricTab) {
      case "success_rate":
        sorted.sort((a, b) => b.pass_rate - a.pass_rate);
        break;
      case "speed":
        sorted.sort((a, b) => a.total_wall_time_ms - b.total_wall_time_ms);
        break;
      case "cost":
        sorted.sort((a, b) => a.total_cost_usd - b.total_cost_usd);
        break;
      case "value":
        sorted.sort((a, b) => b.value_score - a.value_score);
        break;
    }
    return sorted;
  }, [filteredRuns, metricTab]);

  return {
    data,
    loading,
    error,
    filteredRuns,
    sortedRuns,
    runsByModel,
    metricTab,
    setMetricTab,
    viewMode,
    setViewMode,
    filters,
    setFilters,
  };
}
