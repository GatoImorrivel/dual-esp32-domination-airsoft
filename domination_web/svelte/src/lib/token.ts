export interface Token {
  raw: string;
  expiresAt: number;
  ip: string;
  hash: string;
}

export const TOKEN_REFRESH_MARGIN_SECS = 30;

/**
 * Decodes a token from the ESP32S3.
 * Token format: base64_url_safe(hash|expires_at|ip)
 */
export function decodeToken(token: string): Token {
  try {
    const decoded = atob(token.replace(/-/g, "+").replace(/_/g, "/"));

    const parts = decoded.split("|");
    if (parts.length !== 3) {
      throw new Error("Invalid token format");
    }

    const [hash, expiresAtStr, ip] = parts;
    const expiresAt = parseInt(expiresAtStr, 10);

    if (isNaN(expiresAt)) {
      throw new Error("Invalid expiration timestamp");
    }

    return {
      raw: token,
      expiresAt,
      ip,
      hash,
    };
  } catch (error) {
    throw new Error(
      `Failed to decode token: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

export function isTokenExpired(token: Token): boolean {
  const currentTime = Math.floor(Date.now() / 1000);
  return token.expiresAt <= currentTime;
}

export function shouldRefresh(
  token: Token,
  marginSecs = TOKEN_REFRESH_MARGIN_SECS
): boolean {
  const currentTime = Math.floor(Date.now() / 1000);
  return token.expiresAt - currentTime < marginSecs;
}
