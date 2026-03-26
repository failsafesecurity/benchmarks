import type { Model } from "../data/types";

interface Props {
  models: Model[];
  selected: string | null;
  onChange: (modelId: string | null) => void;
}

export default function ModelFilter({ models, selected, onChange }: Props) {
  return (
    <select
      value={selected ?? ""}
      onChange={(e) => onChange(e.target.value || null)}
      className="bg-gray-900 border border-gray-700 rounded-md px-3 py-1.5 text-sm text-gray-200 focus:outline-none focus:ring-1 focus:ring-orange-500"
    >
      <option value="">All Models</option>
      {models.map((m) => (
        <option key={m.id} value={m.id}>
          {m.name}
        </option>
      ))}
    </select>
  );
}
