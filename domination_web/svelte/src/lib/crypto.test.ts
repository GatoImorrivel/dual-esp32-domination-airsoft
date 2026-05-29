import { describe, expect, it } from "vitest";
import {
  hashPasswordWithNonce,
  PASSWORD_NONCE_HASH_VECTOR,
} from "./crypto";

describe("crypto", () => {
  it("matches Rust PASSWORD_NONCE_HASH_VECTOR", async () => {
    const hash = await hashPasswordWithNonce("1234", "test-nonce-fixed");
    expect(hash).toBe(PASSWORD_NONCE_HASH_VECTOR);
  });
});
