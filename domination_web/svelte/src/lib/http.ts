export const BASE_URL = "http://sandi-dominacao.local"

export async function post<B, R>(route: string, body?: B, headers?: Record<string, string>, baseUrl = BASE_URL): Promise<R> {
  const noStartSlashesRoute = route.replace(/^\/+/, "");
  const noTraillingSlashesBaseUrl = baseUrl.replace(/\/+$/, "");

  if (baseUrl === BASE_URL && !body) {
    body = {} as any;
  }

  const request = await fetch(`${noTraillingSlashesBaseUrl}/${noStartSlashesRoute}`, {
    body: JSON.stringify(body),
    headers,
    method: "POST"
  });

  const response = await request.text();

  if (request.headers.get("Content-Type") === "application/json") {
    try {
      const json = JSON.parse(response);
      return json as R;
    } catch (e) {
      console.error({
        cause: e,
        response,
        request
      });
      throw e;
    }
  }

  return response as R;
}

export async function get<R>(route: string, headers?: Record<string, string>, baseUrl = BASE_URL): Promise<R> {
  const noStartSlashesRoute = route.replace(/^\/+/, "");
  const noTraillingSlashesBaseUrl = baseUrl.replace(/\/+$/, "");

  const request = await fetch(`${noTraillingSlashesBaseUrl}/${noStartSlashesRoute}`, {
    headers,
    method: "GET"
  });

  const response = await request.json();

  return response as R;
}
