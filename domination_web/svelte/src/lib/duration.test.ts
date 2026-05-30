import { describe, expect, it } from "vitest";
import {
  clampDurationSecs,
  formatDurationRange,
  formatDurationSecs,
  formatHmsPad,
  hmsToSecs,
  secsToHms,
} from "./duration";

describe("duration", () => {
  it("formats seconds under one hour", () => {
    expect(formatDurationSecs(90)).toBe("1:30");
    expect(formatDurationSecs(45)).toBe("0:45");
  });

  it("formats seconds at or above one hour", () => {
    expect(formatDurationSecs(3661)).toBe("1:01:01");
  });

  it("converts secs to HMS and back", () => {
    expect(secsToHms(330)).toEqual({ hours: 0, minutes: 5, seconds: 30 });
    expect(hmsToSecs(0, 5, 30)).toBe(330);
  });

  it("clamps duration to minimum", () => {
    expect(clampDurationSecs(0)).toBe(1);
    expect(clampDurationSecs(10)).toBe(10);
  });

  it("formats HMS with padding", () => {
    expect(formatHmsPad(0, 5, 30)).toBe("00:05:30");
  });

  it("formats duration range from milliseconds", () => {
    expect(formatDurationRange(45_000, 600_000)).toBe("0:45 / 10:00");
  });
});
