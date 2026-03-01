/// 経過時間を人間が読める形式に変換
export function formatTimeAgo(isoString: string): string {
  if (!isoString) return "";
  const now = Date.now();
  const then = new Date(isoString).getTime();
  const diffMs = now - then;

  if (diffMs < 0) return "just now";

  const seconds = Math.floor(diffMs / 1000);
  if (seconds < 60) return `${seconds}s ago`;

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;

  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;

  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

/// トークン数を読みやすい形式に変換
export function formatTokens(count: number): string {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  }
  if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}K`;
  }
  return count.toString();
}

/// 期間をフォーマット
export function formatDuration(startIso: string, endIso: string): string {
  if (!startIso || !endIso) return "";
  const start = new Date(startIso).getTime();
  const end = new Date(endIso).getTime();
  const diffMs = end - start;

  if (diffMs < 0) return "";

  const minutes = Math.floor(diffMs / 60_000);
  if (minutes < 60) return `${minutes}m`;

  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return `${hours}h ${remainingMinutes}m`;
}
