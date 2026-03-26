import type { MetricTab } from "../data/types";

const TABS: { id: MetricTab; label: string }[] = [
  { id: "success_rate", label: "Success Rate" },
  { id: "speed", label: "Speed" },
  { id: "cost", label: "Cost" },
  { id: "value", label: "Value" },
  { id: "graphs", label: "Graphs" },
];

interface Props {
  active: MetricTab;
  onChange: (tab: MetricTab) => void;
}

export default function MetricTabs({ active, onChange }: Props) {
  return (
    <div className="flex gap-1 bg-gray-900 rounded-lg p-1">
      {TABS.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onChange(tab.id)}
          className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
            active === tab.id
              ? "bg-gray-700 text-white"
              : "text-gray-400 hover:text-gray-200"
          }`}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
