const FRAMEWORK_COLORS: Record<string, string> = {
  ironclaw: "#f97316", // orange
  openclaw: "#22c55e", // green
  nanobot: "#6366f1", // indigo
};

const PALETTE = [
  "#f97316",
  "#22c55e",
  "#6366f1",
  "#ec4899",
  "#14b8a6",
  "#eab308",
  "#8b5cf6",
  "#ef4444",
];

export function frameworkColor(frameworkId: string): string {
  if (FRAMEWORK_COLORS[frameworkId]) return FRAMEWORK_COLORS[frameworkId];
  // Hash-based fallback
  let hash = 0;
  for (let i = 0; i < frameworkId.length; i++) {
    hash = (hash * 31 + frameworkId.charCodeAt(i)) | 0;
  }
  return PALETTE[Math.abs(hash) % PALETTE.length];
}
