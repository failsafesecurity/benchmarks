import type { Suite } from "../data/types";

interface Props {
  suites: Suite[];
  selected: string | null;
  onChange: (suiteId: string | null) => void;
}

export default function SuiteFilter({ suites, selected, onChange }: Props) {
  return (
    <select
      value={selected ?? ""}
      onChange={(e) => onChange(e.target.value || null)}
      className="bg-gray-900 border border-gray-700 rounded-md px-3 py-1.5 text-sm text-gray-200 focus:outline-none focus:ring-1 focus:ring-orange-500"
    >
      <option value="">All Suites</option>
      {suites.map((s) => (
        <option key={s.id} value={s.id}>
          {s.name}
        </option>
      ))}
    </select>
  );
}
