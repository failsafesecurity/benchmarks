import type { LeaderboardData } from "./types";

let cachedData: LeaderboardData | null = null;

export async function loadLeaderboardData(): Promise<LeaderboardData> {
  if (cachedData) return cachedData;

  const resp = await fetch("./data/leaderboard.json");
  if (!resp.ok) {
    throw new Error(`Failed to load leaderboard data: ${resp.status}`);
  }
  cachedData = (await resp.json()) as LeaderboardData;
  return cachedData;
}

export function computeValueScore(
  passRate: number,
  costUsd: number,
): number {
  return (passRate * 1000) / Math.max(costUsd, 0.001);
}
