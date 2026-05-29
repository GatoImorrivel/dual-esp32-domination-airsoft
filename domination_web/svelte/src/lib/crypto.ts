import { sha512 } from "js-sha512";

/** SHA-512(password+nonce) base64url — must match domination_auth::PASSWORD_NONCE_HASH_VECTOR */
export const PASSWORD_NONCE_HASH_VECTOR =
  "o7cndA46XRWj_WVFQ41iFdK_JHdjBD7hNjvk023on4mUwEQLa_GU-hbJQUv8lW9xLIAMmTiOukTLMudtWoMtYQ";

function sha512PureJs(message: string): Uint8Array {
  return new Uint8Array(sha512.array(message));
}

export async function sha512Hash(message: string): Promise<Uint8Array> {
  const hasSubtle =
    typeof globalThis.crypto !== "undefined" &&
    globalThis.crypto.subtle != null;
  const isSecureContext =
    typeof globalThis.isSecureContext === "boolean"
      ? globalThis.isSecureContext
      : false;

  if (hasSubtle && isSecureContext) {
    const encoded = new TextEncoder().encode(message);
    const hashBuffer = await crypto.subtle.digest("SHA-512", encoded);
    return new Uint8Array(hashBuffer);
  }

  return sha512PureJs(message);
}

export function toBase64UrlSafe(bytes: Uint8Array): string {
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  const base64 = btoa(binary);
  return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}

export async function hashPasswordWithNonce(
  password: string,
  nonce: string
): Promise<string> {
  const hashBytes = await sha512Hash(`${password}+${nonce}`);
  return toBase64UrlSafe(hashBytes);
}
