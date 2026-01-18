
import { PublicKey } from "@solana/web3.js";

export function registryPda(programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("signia:registry")],
    programId
  );
}
