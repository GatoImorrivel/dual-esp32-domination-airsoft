import { login } from "./api";
import { decodeToken, isTokenExpired, type Token } from "./token";

const CREDS_KEY = "domination_admin_creds";
const TOKEN_KEY = "domination_admin_token";

export interface StoredCreds {
  username: string;
  password: string;
}

export function loadCreds(): StoredCreds | null {
  try {
    const raw = localStorage.getItem(CREDS_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as StoredCreds;
    if (!parsed.username || !parsed.password) return null;
    return parsed;
  } catch {
    return null;
  }
}

export function saveCreds(creds: StoredCreds): void {
  localStorage.setItem(CREDS_KEY, JSON.stringify(creds));
}

export function clearCreds(): void {
  localStorage.removeItem(CREDS_KEY);
}

function loadTokenRaw(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

function saveTokenRaw(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

function clearTokenRaw(): void {
  localStorage.removeItem(TOKEN_KEY);
}

/** Drop cached token so the next request performs a fresh login. */
export function invalidateStoredToken(): void {
  clearTokenRaw();
}

export function clearSession(): void {
  clearCreds();
  clearTokenRaw();
}

/** True when saved credentials exist (does not verify server session). */
export function isLoggedIn(): boolean {
  return loadCreds() !== null;
}

/** Validates token with the device (refreshes if needed). Clears session on failure. */
export async function verifyServerSession(): Promise<boolean> {
  if (!loadCreds()) {
    return false;
  }
  try {
    const { authorizedGet } = await import("./http");
    await authorizedGet<{ ok: boolean }>("/auth/session");
    return true;
  } catch {
    clearSession();
    return false;
  }
}

export function getStoredToken(): Token | null {
  const raw = loadTokenRaw();
  if (!raw) return null;
  try {
    return decodeToken(raw);
  } catch {
    return null;
  }
}

export async function getValidToken(): Promise<string> {
  const creds = loadCreds();
  if (!creds) {
    throw new Error("Não autenticado");
  }

  const stored = getStoredToken();
  if (stored && !isTokenExpired(stored)) {
    return stored.raw;
  }

  const { token } = await login(creds.username, creds.password);
  saveTokenRaw(token);
  return token;
}

export async function loginAndStore(
  username: string,
  password: string
): Promise<void> {
  const { token } = await login(username, password);
  saveCreds({ username, password });
  saveTokenRaw(token);
}
