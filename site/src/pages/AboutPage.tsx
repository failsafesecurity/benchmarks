export default function AboutPage() {
  return (
    <div className="max-w-3xl mx-auto space-y-8">
      <h1 className="text-2xl font-bold">About Claw Bench</h1>

      <section className="space-y-3">
        <h2 className="text-lg font-semibold">Overview</h2>
        <p className="text-gray-300 leading-relaxed">
          Claw Bench is a multi-framework benchmarking platform for AI coding
          agents. It compares frameworks like IronClaw, OpenClaw, and NanoBot
          across the same tasks and models, providing an apples-to-apples
          comparison of agent capabilities.
        </p>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg font-semibold">Methodology</h2>
        <p className="text-gray-300 leading-relaxed">
          Each framework runs through the same benchmark suites (Spot Checks,
          Trajectory, SWE-Bench, etc.) with identical task definitions and
          scoring criteria. Results capture pass rate, average score, cost,
          and wall-clock time.
        </p>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg font-semibold">Metrics</h2>
        <div className="space-y-2 text-gray-300">
          <div>
            <strong className="text-gray-100">Pass Rate</strong> &mdash;
            Percentage of tasks scored {"\u2265"}1.0 (binary pass/fail).
          </div>
          <div>
            <strong className="text-gray-100">Avg Score</strong> &mdash;
            Mean score across all tasks (0.0&ndash;1.0).
          </div>
          <div>
            <strong className="text-gray-100">Cost</strong> &mdash; Total
            estimated LLM API cost in USD.
          </div>
          <div>
            <strong className="text-gray-100">Time</strong> &mdash; Total
            wall-clock execution time.
          </div>
          <div>
            <strong className="text-gray-100">Value Score</strong> &mdash;
            Derived metric: <code>(pass_rate &times; 1000) / max(cost, $0.001)</code>.
            Higher is better. Rewards high accuracy at low cost.
          </div>
        </div>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg font-semibold">Scoring</h2>
        <p className="text-gray-300 leading-relaxed">
          Tasks are scored using a centralized scoring system that evaluates
          agent responses against predefined assertions (exact match, regex,
          contains, tool usage, etc.). Scoring is applied uniformly regardless
          of which framework produced the response.
        </p>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg font-semibold">Contributing</h2>
        <p className="text-gray-300 leading-relaxed">
          To submit results for a new framework, implement a harness that
          outputs the standard{" "}
          <code className="text-orange-300">run.json</code> +{" "}
          <code className="text-orange-300">tasks.jsonl</code> format with
          the required framework metadata fields. See the repository README
          for details.
        </p>
      </section>
    </div>
  );
}
