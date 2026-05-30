import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

beforeEach(() => {
  vi.stubGlobal(
    "fetch",
    vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => [
        {
          ssid: "Home",
          rssi: -45,
          auth: "WPA2",
          requires_password: true,
        },
      ],
    })
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("wifi API client", () => {
  it("scanWifiNetworks calls GET /app/wifi/scan", async () => {
    const fetchMock = vi.mocked(fetch);
    const { scanWifiNetworks } = await import("./wifi");
    const networks = await scanWifiNetworks();

    expect(fetchMock).toHaveBeenCalledOnce();
    expect(String(fetchMock.mock.calls[0][0])).toContain("/app/wifi/scan");
    expect(networks[0].ssid).toBe("Home");
  });
});
