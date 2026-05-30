export type ApiDuration = { secs: number; nanos: number };

export function durationToMs(d: ApiDuration | null | undefined): number {
  if (!d) return 0;
  return d.secs * 1000 + d.nanos / 1_000_000;
}

export function clampDurationSecs(secs: number, min = 1): number {
  return Math.max(min, Math.floor(secs));
}

export function formatDurationSecs(totalSecs: number): string {
  const s = Math.max(0, Math.floor(totalSecs));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  if (h > 0) {
    return `${h}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
  }
  return `${m}:${String(sec).padStart(2, "0")}`;
}

export function formatDurationMs(ms: number): string {
  return formatDurationSecs(Math.floor(ms / 1000));
}

export function formatDurationRange(currentMs: number, targetMs: number): string {
  return `${formatDurationMs(currentMs)} / ${formatDurationMs(targetMs)}`;
}

export function secsToHms(totalSecs: number): {
  hours: number;
  minutes: number;
  seconds: number;
} {
  const s = Math.max(0, Math.floor(totalSecs));
  return {
    hours: Math.floor(s / 3600),
    minutes: Math.floor((s % 3600) / 60),
    seconds: s % 60,
  };
}

export function hmsToSecs(hours: number, minutes: number, seconds: number): number {
  return hours * 3600 + minutes * 60 + seconds;
}

export function formatHmsPad(hours: number, minutes: number, seconds: number): string {
  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}
