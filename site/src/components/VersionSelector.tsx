import type { Framework } from "../data/types";

interface Props {
  frameworks: Framework[];
  versions: Record<string, string>;
  onChange: (frameworkId: string, version: string) => void;
  disabled: boolean;
}

export default function VersionSelector({
  frameworks,
  versions,
  onChange,
  disabled,
}: Props) {
  if (disabled) return null;

  return (
    <div className="flex gap-2 flex-wrap">
      {frameworks.map((fw) => (
        <div key={fw.id} className="flex items-center gap-1.5">
          <span className="text-xs text-gray-400">{fw.name}:</span>
          <select
            value={versions[fw.id] ?? ""}
            onChange={(e) => onChange(fw.id, e.target.value)}
            className="bg-gray-900 border border-gray-700 rounded px-2 py-1 text-xs text-gray-200 focus:outline-none focus:ring-1 focus:ring-orange-500"
          >
            <option value="">All versions</option>
            {fw.versions.map((v) => (
              <option key={v} value={v}>
                {v}
              </option>
            ))}
          </select>
        </div>
      ))}
    </div>
  );
}
