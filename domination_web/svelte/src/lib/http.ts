import { getValidToken } from "./auth";

export const BASE_URL = "http://sandi-dominacao.local";

export class UnauthorizedError extends Error {
  constructor(message = "Não autorizado") {
    super(message);
    this.name = "UnauthorizedError";
  }
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

export async function post<R>(
  route: string,
  body?: unknown,
  headers?: Record<string, string>,
  baseUrl = BASE_URL
): Promise<R> {
  if (baseUrl === BASE_URL && body === undefined) {
    body = {};
  }

  const request = await fetch(buildUrl(route, baseUrl), {
    body: JSON.stringify(body),
    headers: jsonHeaders(headers),
    method: "POST",
  });

  const response = await request.text();

  if (request.status === 401 || request.status === 403) {
    throw new UnauthorizedError();
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
  baseUrl = BASE_URL
): Promise<R> {
  const request = await fetch(buildUrl(route, baseUrl), {
    headers,
    method: "GET",
  });

  if (request.status === 401 || request.status === 403) {
    throw new UnauthorizedError();
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
  baseUrl = BASE_URL
): Promise<R> {
  const headers = await authHeaders();
  return post<R>(route, body, headers, baseUrl);
}

export async function authorizedGet<R>(
  route: string,
  baseUrl = BASE_URL
): Promise<R> {
  const headers = await authHeaders();
  return get<R>(route, headers, baseUrl);
}
