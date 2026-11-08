/**
 * Runs one full pool lifecycle against a local validator and records, per transaction,
 * the raw program logs, the self-CPI bytes and the decoded `emit_cpi!` event.
 *
 * Usage (validator running with the program deployed):
 *   ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 ANCHOR_WALLET=~/.config/solana/id.json \
 *     npx ts-node --transpile-only scripts/event-logs-example.ts [output.md]
 */
import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotent,
  createMint,
  getAssociatedTokenAddressSync,
  mintTo,
} from "@solana/spl-token";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { createHash } from "crypto";
import { writeFileSync } from "fs";
import idl from "../target/idl/amm.json";
import { Amm } from "../target/types/amm";

// `EVENT_IX_TAG_LE`: the hash prefix is read as a big-endian u64 and serialized little-endian.
const EVENT_IX_TAG = Buffer.from(
  createHash("sha256").update("anchor:event").digest().subarray(0, 8)
).reverse();

function toPlain(value: unknown): unknown {
  if (value instanceof PublicKey) return value.toBase58();
  if (BN.isBN(value)) return value.toString();
  if (value === null || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value).map(([key, inner]) => [key, toPlain(inner)])
  );
}

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = new Program(idl as Amm, provider);
  const { connection } = provider;
  const payer = (provider.wallet as anchor.Wallet).payer;
  const user = payer.publicKey;
  const output = process.argv[2] ?? "doc/ejemplo-logs-eventos.md";
  const sections: string[] = [];

  async function record(title: string, signature: string) {
    const latest = await connection.getLatestBlockhash();
    await connection.confirmTransaction({ signature, ...latest }, "confirmed");
    const tx = await connection.getTransaction(signature, {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0,
    });
    const meta = tx!.meta!;
    const keys = tx!.transaction.message.staticAccountKeys;
    const rawEvents: string[] = [];
    const events: { name: string; data: unknown }[] = [];
    for (const inner of meta.innerInstructions ?? []) {
      for (const ix of inner.instructions) {
        if (!keys[ix.programIdIndex].equals(program.programId)) continue;
        const data = Buffer.from(anchor.utils.bytes.bs58.decode(ix.data));
        if (!data.subarray(0, 8).equals(EVENT_IX_TAG)) continue;
        rawEvents.push(data.toString("hex"));
        const event = program.coder.events.decode(
          anchor.utils.bytes.base64.encode(data.subarray(8))
        );
        if (event) events.push({ name: event.name, data: toPlain(event.data) });
      }
    }

    sections.push(
      [
        `## ${title}`,
        "",
        `Firma: \`${signature}\` · compute units: ${meta.computeUnitsConsumed}`,
        "",
        "### Logs del programa (`meta.logMessages`)",
        "",
        "```text",
        ...(meta.logMessages ?? []),
        "```",
        "",
        "### Bytes de la self-CPI (`meta.innerInstructions`, hex)",
        "",
        "```text",
        ...rawEvents,
        "```",
        "",
        "### Evento decodificado (`program.coder.events.decode`)",
        "",
        "```json",
        JSON.stringify(events, null, 2),
        "```",
        "",
      ].join("\n")
    );
    console.log(`${title}: ${events.map((e) => e.name).join(", ")}`);
  }

  const mintX = await createMint(connection, payer, user, null, 6);
  const mintY = await createMint(connection, payer, user, null, 6);
  const seed = new BN(Date.now());
  const [config] = PublicKey.findProgramAddressSync(
    [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
    program.programId
  );
  const [mintLp] = PublicKey.findProgramAddressSync(
    [Buffer.from("lp"), config.toBuffer()],
    program.programId
  );
  const vaultX = getAssociatedTokenAddressSync(mintX, config, true);
  const vaultY = getAssociatedTokenAddressSync(mintY, config, true);
  const lockLp = getAssociatedTokenAddressSync(mintLp, config, true);
  const userLp = getAssociatedTokenAddressSync(mintLp, user);

  await record(
    "initialize (fee 30 bps, con admin)",
    await program.methods
      .initialize(seed, 30, user)
      .accountsPartial({
        initializer: user,
        mintX,
        mintY,
        config,
        mintLp,
        vaultX,
        vaultY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
  );

  const userX = await createAssociatedTokenAccountIdempotent(
    connection,
    payer,
    mintX,
    user
  );
  const userY = await createAssociatedTokenAccountIdempotent(
    connection,
    payer,
    mintY,
    user
  );
  await mintTo(connection, payer, mintX, userX, payer, 2_000_000);
  await mintTo(connection, payer, mintY, userY, payer, 2_000_000);

  const poolAccounts = {
    user,
    mintX,
    mintY,
    config,
    vaultX,
    vaultY,
    userX,
    userY,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
  const depositAccounts = {
    ...poolAccounts,
    mintLp,
    userLp,
    lockLp,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  };

  await record(
    "deposit (primer depósito 1:1, bloquea MINIMUM_LIQUIDITY)",
    await program.methods
      .deposit(new BN(1_000_000), new BN(1_000_000), new BN(999_000))
      .accountsPartial(depositAccounts)
      .rpc()
  );
  await record(
    "deposit desbalanceado (ofrece 100k X + 500k Y; solo se cobra la proporción)",
    await program.methods
      .deposit(new BN(100_000), new BN(500_000), new BN(1))
      .accountsPartial(depositAccounts)
      .rpc()
  );
  await record(
    "swap X→Y (10k de entrada)",
    await program.methods
      .swap(true, new BN(10_000), new BN(1))
      .accountsPartial(poolAccounts)
      .rpc()
  );
  await record(
    "withdraw (quema 100k LP)",
    await program.methods
      .withdraw(new BN(100_000), new BN(1), new BN(1))
      .accountsPartial({ ...poolAccounts, mintLp, userLp })
      .rpc()
  );
  await record(
    "set_locked(true)",
    await program.methods
      .setLocked(true)
      .accountsPartial({ authority: user, config })
      .rpc()
  );
  await record(
    "set_authority(None) — renuncia irreversible",
    await program.methods
      .setAuthority(null)
      .accountsPartial({ authority: user, config })
      .rpc()
  );

  const header = [
    "# Ejemplo de logs y eventos (localnet)",
    "",
    `> Generado con \`scripts/event-logs-example.ts\` el ${new Date().toISOString()} contra \`solana-test-validator\`. Pool: \`${config.toBase58()}\`.`,
    "",
    "Cómo leerlo:",
    "",
    `- **Logs:** cada transacción termina con \`Program ${program.programId.toBase58()} invoke [2]\`. Esa es la self-CPI del evento: el evento no aparece como texto en los logs.`,
    "- **Bytes de la self-CPI:** los datos de esa instrucción interna. Son 8 bytes de `EVENT_IX_TAG_LE` (`e445a52e51cb9a1d`), luego 8 bytes de discriminador del evento y después los campos en Borsh.",
    "- **Evento decodificado:** lo que devuelve `program.coder.events.decode` (montos en unidades mínimas; el mint tiene 6 decimales).",
    "",
    "Para regenerarlo: levantar `solana-test-validator` con el programa (o `anchor localnet`) y correr el comando del encabezado del script.",
    "",
  ].join("\n");
  writeFileSync(output, `${header}\n${sections.join("\n")}`);
  console.log(`written ${output}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
