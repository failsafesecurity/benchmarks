import { useLeaderboardData } from "../hooks/useLeaderboardData";
import MetricTabs from "../components/MetricTabs";
import ModelFilter from "../components/ModelFilter";
import SuiteFilter from "../components/SuiteFilter";
import VersionSelector from "../components/VersionSelector";
import FrameworkCard from "../components/FrameworkCard";
import LeaderboardTable from "../components/LeaderboardTable";
import GroupedBarChart from "../components/charts/GroupedBarChart";
import ScatterPlot from "../components/charts/ScatterPlot";
import TrendChart from "../components/charts/TrendChart";

export default function LeaderboardPage() {
  const {
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
  } = useLeaderboardData();

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="text-gray-400">Loading benchmark data...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="text-red-400">Error: {error}</div>
      </div>
    );
  }

  if (!data) return null;

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center gap-4">
        <h1 className="text-2xl font-bold">Leaderboard</h1>
        <div className="flex-1" />
        <div className="flex flex-wrap items-center gap-2">
          <ModelFilter
            models={data.models}
            selected={filters.modelId}
            onChange={(modelId) =>
              setFilters((f) => ({ ...f, modelId }))
            }
          />
          <SuiteFilter
            suites={data.suites}
            selected={filters.suiteId}
            onChange={(suiteId) =>
              setFilters((f) => ({ ...f, suiteId }))
            }
          />
          <label className="flex items-center gap-1.5 text-sm text-gray-400">
            <input
              type="checkbox"
              checked={filters.latestOnly}
              onChange={(e) =>
                setFilters((f) => ({
                  ...f,
                  latestOnly: e.target.checked,
                }))
              }
              className="rounded border-gray-600 bg-gray-800 text-orange-500 focus:ring-orange-500"
            />
            Latest only
          </label>
        </div>
      </div>

      {!filters.latestOnly && (
        <VersionSelector
          frameworks={data.frameworks}
          versions={filters.frameworkVersions}
          onChange={(fwId, version) =>
            setFilters((f) => ({
              ...f,
              frameworkVersions: {
                ...f.frameworkVersions,
                [fwId]: version,
              },
            }))
          }
          disabled={filters.latestOnly}
        />
      )}

      <div className="flex flex-col sm:flex-row sm:items-center gap-4">
        <MetricTabs active={metricTab} onChange={setMetricTab} />
        <div className="flex-1" />
        {metricTab !== "graphs" && (
          <div className="flex gap-1 bg-gray-900 rounded-lg p-1">
            <button
              onClick={() => setViewMode("card")}
              className={`px-3 py-1.5 rounded-md text-sm font-medium ${
                viewMode === "card"
                  ? "bg-gray-700 text-white"
                  : "text-gray-400 hover:text-gray-200"
              }`}
            >
              Cards
            </button>
            <button
              onClick={() => setViewMode("table")}
              className={`px-3 py-1.5 rounded-md text-sm font-medium ${
                viewMode === "table"
                  ? "bg-gray-700 text-white"
                  : "text-gray-400 hover:text-gray-200"
              }`}
            >
              Table
            </button>
          </div>
        )}
      </div>

      {metricTab === "graphs" ? (
        <div className="space-y-8">
          <section>
            <h2 className="text-lg font-semibold mb-3">
              Pass Rate by Model & Framework
            </h2>
            <div className="bg-gray-900 rounded-xl p-4 border border-gray-800">
              <GroupedBarChart
                runs={filteredRuns}
                data={data}
                metric="pass_rate"
              />
            </div>
          </section>
          <section>
            <h2 className="text-lg font-semibold mb-3">
              Cost vs Accuracy
            </h2>
            <div className="bg-gray-900 rounded-xl p-4 border border-gray-800">
              <ScatterPlot runs={filteredRuns} data={data} />
            </div>
          </section>
          <section>
            <h2 className="text-lg font-semibold mb-3">
              Pass Rate Trend
            </h2>
            <div className="bg-gray-900 rounded-xl p-4 border border-gray-800">
              <TrendChart runs={filteredRuns} data={data} />
            </div>
          </section>
        </div>
      ) : viewMode === "table" ? (
        <div className="bg-gray-900 rounded-xl border border-gray-800">
          <LeaderboardTable runs={sortedRuns} data={data} />
        </div>
      ) : (
        <div className="space-y-8">
          {[...runsByModel.entries()].map(([modelId, runs]) => {
            const model = data.models.find((m) => m.id === modelId);
            return (
              <section key={modelId}>
                <h2 className="text-lg font-semibold mb-3">
                  {model?.name ?? modelId}
                  {model?.provider && (
                    <span className="text-sm font-normal text-gray-500 ml-2">
                      {model.provider}
                    </span>
                  )}
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {[...runs]
                    .sort((a, b) => b.pass_rate - a.pass_rate)
                    .map((run) => (
                      <FrameworkCard
                        key={run.run_id}
                        run={run}
                        framework={data.frameworks.find(
                          (f) => f.id === run.framework_id,
                        )}
                      />
                    ))}
                </div>
              </section>
            );
          })}
          {runsByModel.size === 0 && (
            <p className="text-center text-gray-500 py-8">
              No runs match the current filters.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
