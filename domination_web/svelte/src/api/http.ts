export async function post<B, R>(route: string, body: B, headers?: Record<string, string>): Promise<R> {
  const request = await fetch(route, {
    body: JSON.stringify(body),
    headers,
    method: "POST"
  });

  const response = await request.json();

  return response as R;
}

export async function get<R>(route: string, headers?: Record<string, string>): Promise<R> {
  const request = await fetch(route, {
    headers,
    method: "GET"
  });

  const response = await request.json();

  return response as R;
}
