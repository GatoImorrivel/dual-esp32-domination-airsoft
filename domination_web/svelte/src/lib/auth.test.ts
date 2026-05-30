import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  clearSession,
  getValidToken,
  loadCreds,
  loginAndStore,
  saveCreds,
} from "./auth";
import { PASSWORD_NONCE_HASH_VECTOR } from "./crypto";

const storage = new Map<string, string>();

function jsonResponse(body: unknown) {
  return {
    ok: true,
    status: 200,
    headers: { get: () => "application/json" },
    text: async () => JSON.stringify(body),
  };
}

function mockApiFetch(handlers: {
  challenge?: unknown;
  login?: unknown;
}) {
  return vi.fn((url: string | URL, init?: RequestInit) => {
    const href = String(url);
    if (href.includes("/auth/challenge")) {
      return Promise.resolve(
        jsonResponse(handlers.challenge ?? { nonce: "test-nonce-fixed" })
      );
    }
    if (href.includes("/auth/login")) {
      return Promise.resolve(
        jsonResponse(
          handlers.login ?? {
            token: btoa("tok|9999999999|10.0.0.5")
              .replace(/\+/g, "-")
              .replace(/\//g, "_")
              .replace(/=/g, ""),
          }
        )
      );
    }
    return Promise.reject(new Error(`unexpected fetch: ${href}`));
  });
}

beforeEach(() => {
  storage.clear();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => storage.get(k) ?? null,
    setItem: (k: string, v: string) => storage.set(k, v),
    removeItem: (k: string) => storage.delete(k),
    clear: () => storage.clear(),
  });
  clearSession();
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("auth session", () => {
  it("loginAndStore saves creds and token via fetch", async () => {
    const fetchMock = mockApiFetch({});
    vi.stubGlobal("fetch", fetchMock);

    await loginAndStore("root", "1234");

    expect(loadCreds()).toEqual({ username: "root", password: "1234" });

    const loginCall = fetchMock.mock.calls.find((c) =>
      String(c[0]).includes("/auth/login")
    );
    expect(loginCall).toBeDefined();
    const loginBody = JSON.parse((loginCall![1] as RequestInit).body as string);
    expect(loginBody.password).toBe(PASSWORD_NONCE_HASH_VECTOR);

    expect(storage.get("domination_admin_token")).toBeTruthy();
  });

  it("loginAndStore does not save creds when login fails", async () => {
    const fetchMock = vi.fn((url: string | URL) => {
      const href = String(url);
      if (href.includes("/auth/challenge")) {
        return Promise.resolve(jsonResponse({ nonce: "n" }));
      }
      if (href.includes("/auth/login")) {
        return Promise.resolve({
          ok: false,
          status: 401,
          headers: { get: () => "text/plain" },
          text: async () => "unauthorized",
        });
      }
      return Promise.reject(new Error(`unexpected fetch: ${href}`));
    });
    vi.stubGlobal("fetch", fetchMock);

    await expect(loginAndStore("root", "wrong")).rejects.toThrow();
    expect(loadCreds()).toBeNull();
    expect(storage.get("domination_admin_token")).toBeFalsy();
  });

  it("getValidToken refreshes when near expiry", async () => {
    saveCreds({ username: "root", password: "1234" });
    const nearExpiry = Math.floor(Date.now() / 1000) + 10;
    const staleToken = btoa(`h|${nearExpiry}|10.0.0.1`)
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=/g, "");
    storage.set("domination_admin_token", staleToken);

    const newExpiry = Math.floor(Date.now() / 1000) + 120;
    const freshToken = btoa(`h|${newExpiry}|10.0.0.1`)
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=/g, "");

    const fetchMock = mockApiFetch({
      challenge: { nonce: "n2" },
      login: { token: freshToken },
    });
    vi.stubGlobal("fetch", fetchMock);

    const token = await getValidToken();
    expect(token).toBe(freshToken);
  });
});
