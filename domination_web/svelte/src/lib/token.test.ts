import { describe, expect, it, vi, afterEach } from "vitest";
import {
  decodeToken,
  isTokenExpired,
  shouldRefresh,
  TOKEN_REFRESH_MARGIN_SECS,
} from "./token";

/** base64url("abc|9999999999|192.168.1.1") */
const SAMPLE_TOKEN = btoa("abc|9999999999|192.168.1.1")
  .replace(/\+/g, "-")
  .replace(/\//g, "_")
  .replace(/=/g, "");

describe("token", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("decodes hash, expiry, and ip", () => {
    const token = decodeToken(SAMPLE_TOKEN);
    expect(token.hash).toBe("abc");
    expect(token.expiresAt).toBe(9999999999);
    expect(token.ip).toBe("192.168.1.1");
    expect(token.raw).toBe(SAMPLE_TOKEN);
  });

  it("detects expiry", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date((9999999999 - 1) * 1000));
    const token = decodeToken(SAMPLE_TOKEN);
    expect(isTokenExpired(token)).toBe(false);
    vi.setSystemTime(new Date((9999999999 + 1) * 1000));
    expect(isTokenExpired(token)).toBe(true);
  });

  it("shouldRefresh near expiry", () => {
    vi.useFakeTimers();
    const expiresAt = 1_000_000;
    const raw = btoa(`h|${expiresAt}|10.0.0.1`)
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=/g, "");
    const token = decodeToken(raw);

    vi.setSystemTime(new Date((expiresAt - TOKEN_REFRESH_MARGIN_SECS - 1) * 1000));
    expect(shouldRefresh(token)).toBe(false);

    vi.setSystemTime(new Date((expiresAt - 10) * 1000));
    expect(shouldRefresh(token)).toBe(true);
  });
});
