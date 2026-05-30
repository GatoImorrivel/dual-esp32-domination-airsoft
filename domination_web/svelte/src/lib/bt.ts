import { authorizedGet, authorizedPost } from "./http";

export interface AudioSink {
  address: string;
  name: string | null;
}

export interface BtSinksResponse {
  paired: AudioSink | null;
  discovered: AudioSink[];
  scanning?: boolean;
  connected?: boolean;
}

export function listSinks(): Promise<BtSinksResponse> {
  return authorizedGet<BtSinksResponse>("/bt/sinks");
}

const BT_POLL_INTERVAL_MS = 500;
const BT_SCAN_MAX_ATTEMPTS = 40;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Start async BT scan on the coprocessor (non-blocking). */
export function startBtScan(): Promise<void> {
  return authorizedPost<{ scanning: boolean }>("/bt/scan", {}).then(() => {});
}

/** POST /bt/scan then poll GET /bt/sinks until scan completes. */
export async function scanSinks(): Promise<BtSinksResponse> {
  await startBtScan();

  for (let attempt = 0; attempt < BT_SCAN_MAX_ATTEMPTS; attempt++) {
    if (attempt > 0) {
      await sleep(BT_POLL_INTERVAL_MS);
    }
    const status = await listSinks();
    if (!status.scanning) {
      return status;
    }
  }

  throw new Error("Tempo esgotado ao buscar dispositivos Bluetooth");
}

export function pairSink(address: string): Promise<BtSinksResponse> {
  return authorizedPost<BtSinksResponse>("/bt/pair", { address });
}

export function unpairSink(): Promise<BtSinksResponse> {
  return authorizedPost<BtSinksResponse>("/bt/unpair", {});
}
