import { hashPasswordWithNonce } from "./crypto";
import { post } from "./http";

export async function login(
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
