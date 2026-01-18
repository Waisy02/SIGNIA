import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

export function pdaRegistry(programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from("signia:registry")], programId);
}

export function pdaEntry(programId: PublicKey, namespace: string, schemaHash32: Buffer): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("signia:entry"), Buffer.from(namespace, "utf8"), schemaHash32],
    programId
  );
}

export function hexToBuf32(hex: string): Buffer {
  const b = Buffer.from(hex, "hex");
  if (b.length !== 32) throw new Error("hash must be 32 bytes");
  return b;
}
