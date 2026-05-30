import { getValidToken, clearSession } from "./auth";
import { notifySessionExpired } from "./session";

/** mDNS hostname when the device is already on your LAN (STA mode). */
export const BASE_URL = "http://sandi-dominacao.local";

const MDNS_HOST = "sandi-dominacao";

/** Prefer mDNS hostname; fall back to page origin (e.g. AP gateway IP) for debug. */
export function resolveBaseUrl(override?: string): string {
  if (override) {
    return override;
  }
  if (typeof window !== "undefined") {
    const { hostname, origin } = window.location;
    if (hostname.includes(MDNS_HOST)) {
      return BASE_URL;
    }
    if (origin.startsWith("http://") || origin.startsWith("https://")) {
      return origin;
    }
  }
  return BASE_URL;
}

export class UnauthorizedError extends Error {
  constructor(message = "Não autorizado") {
    super(message);
    this.name = "UnauthorizedError";
  }
}

function throwUnauthorized(): never {
  throw new UnauthorizedError();
}

function handleAuthorizedUnauthorized(): never {
  clearSession();
  notifySessionExpired();
  throwUnauthorized();
}

function buildUrl(route: string, baseUrl: string): string {
  const noStartSlashesRoute = route.replace(/^\/+/, "");
  const noTraillingSlashesBaseUrl = baseUrl.replace(/\/+$/, "");
  return `${noTraillingSlashesBaseUrl}/${noStartSlashesRoute}`;
}

function jsonHeaders(extra?: Record<string, string>): Record<string, string> {
  return {
    "Content-Type": "application/json",
    ...extra,
  };
}

/** POST without waiting for response (STA Wi-Fi configure while link drops). */
export function postFireAndForget(
  route: string,
  body?: unknown,
  headers?: Record<string, string>,
  baseUrl?: string
): void {
  const resolvedBase = resolveBaseUrl(baseUrl);
  void fetch(buildUrl(route, resolvedBase), {
    body: JSON.stringify(body ?? {}),
    headers: jsonHeaders(headers),
    method: "POST",
    keepalive: true,
  }).catch(() => {});
}

export async function post<R>(
  route: string,
  body?: unknown,
  headers?: Record<string, string>,
  baseUrl?: string
): Promise<R> {
  const resolvedBase = resolveBaseUrl(baseUrl);
  if (body === undefined) {
    body = {};
  }

  const request = await fetch(buildUrl(route, resolvedBase), {
    body: JSON.stringify(body),
    headers: jsonHeaders(headers),
    method: "POST",
  });

  const response = await request.text();

  if (request.status === 401 || request.status === 403) {
    throwUnauthorized();
  }

  if (!request.ok) {
    throw new Error(response || `HTTP ${request.status}`);
  }

  if (request.headers.get("Content-Type")?.includes("application/json")) {
    try {
      return JSON.parse(response) as R;
    } catch (e) {
      console.error({ cause: e, response, request });
      throw e;
    }
  }

  return response as R;
}

export async function get<R>(
  route: string,
  headers?: Record<string, string>,
  baseUrl?: string
): Promise<R> {
  const request = await fetch(buildUrl(route, resolveBaseUrl(baseUrl)), {
    headers,
    method: "GET",
  });

  if (request.status === 401 || request.status === 403) {
    throwUnauthorized();
  }

  if (!request.ok) {
    const text = await request.text();
    throw new Error(text || `HTTP ${request.status}`);
  }

  return (await request.json()) as R;
}

async function authHeaders(): Promise<Record<string, string>> {
  const token = await getValidToken();
  return { Authorization: token };
}

export async function authorizedPost<R>(
  route: string,
  body?: unknown,
  baseUrl?: string
): Promise<R> {
  try {
    const headers = await authHeaders();
    return await post<R>(route, body, headers, baseUrl);
  } catch (e) {
    if (e instanceof UnauthorizedError) {
      handleAuthorizedUnauthorized();
    }
    throw e;
  }
}

export async function authorizedGet<R>(
  route: string,
  baseUrl?: string
): Promise<R> {
  try {
    const headers = await authHeaders();
    return await get<R>(route, headers, baseUrl);
  } catch (e) {
    if (e instanceof UnauthorizedError) {
      handleAuthorizedUnauthorized();
    }
    throw e;
  }
}
