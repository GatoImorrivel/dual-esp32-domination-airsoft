import { get, post } from "./http";

export interface WifiNetwork {
  ssid: string;
  rssi: number;
  auth: string;
  requires_password: boolean;
}

export interface WifiScanStatus {
  scanning: boolean;
  networks: WifiNetwork[];
}

const POLL_INTERVAL_MS = 500;
const MAX_POLL_ATTEMPTS = 30;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Start async scan on device, then poll until done or timeout. */
export async function scanWifiNetworks(
  onProgress?: (networks: WifiNetwork[]) => void
): Promise<WifiNetwork[]> {
  await post("/app/wifi/scan", {});

  for (let attempt = 0; attempt < MAX_POLL_ATTEMPTS; attempt++) {
    if (attempt > 0) {
      await sleep(POLL_INTERVAL_MS);
    }
    const status = await get<WifiScanStatus>("/app/wifi/scan");

    if (status.networks.length > 0) {
      onProgress?.(status.networks);
    }
    if (!status.scanning) {
      return status.networks;
    }
  }

  throw new Error("Tempo esgotado ao buscar redes Wi-Fi");
}
