import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { pdaRegistry, pdaEntry, hexToBuf32 } from "./utils.js";

describe("signia-registry", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SigniaRegistry as anchor.Program;

  it("init registry", async () => {
    const [registryPda] = pdaRegistry(program.programId);

    await program.methods
      .initRegistry({ authority: provider.wallet.publicKey })
      .accounts({
        payer: provider.wallet.publicKey,
        registry: registryPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const registry = await program.account.registry.fetch(registryPda);
    expect(registry.authority.toBase58()).to.eq(provider.wallet.publicKey.toBase58());
  });

  it("register schema entry and publish current", async () => {
    const [registryPda] = pdaRegistry(program.programId);

    const namespace = "core-v1";
    const hashHex = "a".repeat(64);
    const hashBuf = hexToBuf32(hashHex);
    const [entryPda] = pdaEntry(program.programId, namespace, hashBuf);

    await program.methods
      .registerSchema({
        namespace,
        schemaHashHex: hashHex,
        kind: "schema",
        uri: "ipfs://example",
        versionTag: "v1",
      })
      .accounts({
        payer: provider.wallet.publicKey,
        authority: provider.wallet.publicKey,
        registry: registryPda,
        entry: entryPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const entry = await program.account.entry.fetch(entryPda);
    expect(entry.namespace).to.eq(namespace);
    expect(Buffer.from(entry.schemaHash as number[]).toString("hex")).to.eq(hashHex);

    await program.methods
      .publishVersion({ namespace, schemaHashHex: hashHex, makeCurrent: true })
      .accounts({
        authority: provider.wallet.publicKey,
        registry: registryPda,
        entry: entryPda,
      })
      .rpc();

    const entry2 = await program.account.entry.fetch(entryPda);
    expect(entry2.current).to.eq(true);
  });

  it("revoke entry", async () => {
    const [registryPda] = pdaRegistry(program.programId);
    const namespace = "core-v1";
    const hashHex = "a".repeat(64);
    const [entryPda] = pdaEntry(program.programId, namespace, hexToBuf32(hashHex));

    await program.methods
      .revokeEntry({ namespace, schemaHashHex: hashHex, revoke: true })
      .accounts({
        authority: provider.wallet.publicKey,
        registry: registryPda,
        entry: entryPda,
      })
      .rpc();

    const entry = await program.account.entry.fetch(entryPda);
    expect(entry.revoked).to.eq(true);
    expect(entry.current).to.eq(false);
  });

  it("transfer authority", async () => {
    const [registryPda] = pdaRegistry(program.programId);

    const kp = anchor.web3.Keypair.generate();

    await program.methods
      .transferAuthority({ newAuthority: kp.publicKey })
      .accounts({ authority: provider.wallet.publicKey, registry: registryPda })
      .rpc();

    const registry = await program.account.registry.fetch(registryPda);
    expect(registry.authority.toBase58()).to.eq(kp.publicKey.toBase58());
  });
});
