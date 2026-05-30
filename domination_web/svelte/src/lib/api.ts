import { hashPasswordWithNonce } from "./crypto";
import { post } from "./http";

/** One challenge+login at a time (parallel calls overwrite the server nonce). */
let loginInFlight: Promise<{ token: string }> | null = null;

async function loginOnce(
  username: string,
  password: string
): Promise<{ token: string }> {
  const challengeResponse = await post<{ nonce: string }>("/auth/challenge", {
    username,
  }).catch((e) => {
    console.error("Nonce failed", { cause: e, username });
    throw new Error("Falha ao autenticar");
  });

  const hashedPassword = await hashPasswordWithNonce(
    password,
    challengeResponse.nonce
  );

  return post<{ token: string }>("/auth/login", {
    username,
    password: hashedPassword,
  }).catch((e) => {
    console.error("Login failed", { cause: e, username });
    throw new Error("Falha ao autenticar");
  });
}

export async function login(
  username: string,
  password: string
): Promise<{ token: string }> {
  if (loginInFlight) {
    return loginInFlight;
  }

  loginInFlight = loginOnce(username, password).finally(() => {
    loginInFlight = null;
  });
  return loginInFlight;
}
