import { authorizedGet, authorizedPost } from "./http";

export interface AudioSink {
  address: string;
  name: string | null;
}

export interface BtSinksResponse {
  paired: AudioSink | null;
  discovered: AudioSink[];
}

export function listSinks(): Promise<BtSinksResponse> {
  return authorizedGet<BtSinksResponse>("/bt/sinks");
}

export function scanSinks(): Promise<BtSinksResponse> {
  return authorizedPost<BtSinksResponse>("/bt/scan", {});
}

export function pairSink(address: string): Promise<BtSinksResponse> {
  return authorizedPost<BtSinksResponse>("/bt/pair", { address });
}

export function unpairSink(): Promise<BtSinksResponse> {
  return authorizedPost<BtSinksResponse>("/bt/unpair", {});
}
