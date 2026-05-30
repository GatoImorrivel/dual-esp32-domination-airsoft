import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const storage = new Map<string, string>();

beforeEach(() => {
  storage.clear();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => storage.get(k) ?? null,
    setItem: (k: string, v: string) => storage.set(k, v),
    removeItem: (k: string) => storage.delete(k),
    clear: () => storage.clear(),
  });
  storage.set(
    "domination_admin_token",
    btoa("h|9999999999|10.0.0.1")
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=/g, "")
  );
  storage.set(
    "domination_admin_creds",
    JSON.stringify({ username: "root", password: "1234" })
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("bt API client", () => {
  it("listSinks calls GET /bt/sinks with Authorization", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        paired: null,
        discovered: [],
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    const { listSinks } = await import("./bt");
    await listSinks();

    expect(fetchMock).toHaveBeenCalledOnce();
    const [url, init] = fetchMock.mock.calls[0];
    expect(String(url)).toContain("/bt/sinks");
    expect((init as RequestInit).method).toBe("GET");
    expect((init as RequestInit).headers).toMatchObject({
      Authorization: expect.any(String),
    });
  });

  it("scanSinks POSTs /bt/scan then polls GET /bt/sinks", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: { get: () => "application/json" },
        text: async () => JSON.stringify({ scanning: true }),
      })
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          paired: null,
          discovered: [{ address: "AA:BB:CC:DD:EE:FF", name: "Test" }],
          scanning: false,
        }),
      });
    vi.stubGlobal("fetch", fetchMock);

    const { scanSinks } = await import("./bt");
    const res = await scanSinks();

    expect(String(fetchMock.mock.calls[0][0])).toContain("/bt/scan");
    expect(String(fetchMock.mock.calls[1][0])).toContain("/bt/sinks");
    expect(res.discovered).toHaveLength(1);
  });
});
