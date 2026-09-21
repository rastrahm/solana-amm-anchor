import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import {
  createMint,
  createAssociatedTokenAccountIdempotent,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { expect } from "chai";
import { Amm } from "../target/types/amm";

const CONFIG_ACCOUNT_SPACE = 150;
const CONFIG_SEED = Buffer.from("config");
const LP_SEED = Buffer.from("lp");
const MINIMUM_LIQUIDITY = 1_000;

describe("phase6: sealevel security", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.amm as Program<Amm>;
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  function errorBlob(err: unknown): string {
    const anchorErr = err as {
      message?: string;
      logs?: string[];
      error?: { errorCode?: { code?: string; number?: number } };
    };
    const logs = (anchorErr.logs ?? []).join("\n");
    const code = anchorErr.error?.errorCode?.code ?? "";
    return `${anchorErr.message ?? ""}\n${logs}\n${code}`;
  }

  async function createPairMints(): Promise<[PublicKey, PublicKey]> {
    const x = await createMint(
      connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    const y = await createMint(
      connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    return [x, y];
  }

  function derivePoolPdas(seed: BN) {
    const [config, configBump] = PublicKey.findProgramAddressSync(
      [CONFIG_SEED, seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const [mintLp, lpBump] = PublicKey.findProgramAddressSync(
      [LP_SEED, config.toBuffer()],
      program.programId
    );
    return { config, configBump, mintLp, lpBump };
  }

  async function initializePool(seed: BN, mintX: PublicKey, mintY: PublicKey, fee = 30) {
    const { config, mintLp } = derivePoolPdas(seed);
    const vaultX = getAssociatedTokenAddressSync(mintX, config, true, TOKEN_PROGRAM_ID);
    const vaultY = getAssociatedTokenAddressSync(mintY, config, true, TOKEN_PROGRAM_ID);

    await program.methods
      .initialize(seed, fee, null)
      .accountsPartial({
        initializer: wallet.publicKey,
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
      .rpc();

    return { config, mintLp, vaultX, vaultY };
  }

  async function fundUser(mintX: PublicKey, mintY: PublicKey, amount: bigint) {
    const userX = await createAssociatedTokenAccountIdempotent(
      connection,
      wallet.payer,
      mintX,
      wallet.publicKey,
      undefined,
      TOKEN_PROGRAM_ID
    );
    const userY = await createAssociatedTokenAccountIdempotent(
      connection,
      wallet.payer,
      mintY,
      wallet.publicKey,
      undefined,
      TOKEN_PROGRAM_ID
    );
    await mintTo(
      connection,
      wallet.payer,
      mintX,
      userX,
      wallet.payer,
      amount,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    await mintTo(
      connection,
      wallet.payer,
      mintY,
      userY,
      wallet.payer,
      amount,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    return { userX, userY };
  }

  async function deposit(
    seed: BN,
    mintX: PublicKey,
    mintY: PublicKey,
    amount: BN,
    minLp: BN,
    overrides: Record<string, PublicKey> = {}
  ) {
    const { config, mintLp } = derivePoolPdas(seed);
    const vaultX = getAssociatedTokenAddressSync(mintX, config, true, TOKEN_PROGRAM_ID);
    const vaultY = getAssociatedTokenAddressSync(mintY, config, true, TOKEN_PROGRAM_ID);
    const userX = getAssociatedTokenAddressSync(
      mintX,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const userY = getAssociatedTokenAddressSync(
      mintY,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const userLp = getAssociatedTokenAddressSync(
      mintLp,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const lockLp = getAssociatedTokenAddressSync(mintLp, config, true, TOKEN_PROGRAM_ID);

    return program.methods
      .deposit(amount, amount, minLp)
      .accountsPartial({
        user: wallet.publicKey,
        mintX,
        mintY,
        mintLp,
        config,
        vaultX,
        vaultY,
        userX,
        userY,
        userLp,
        lockLp,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        ...overrides,
      })
      .rpc();
  }

  it("rent/space: Config account is exactly 8 + InitSpace (150)", async () => {
    const seed = new BN(600);
    const [mintX, mintY] = await createPairMints();
    const { config } = await initializePool(seed, mintX, mintY);
    const info = await connection.getAccountInfo(config);
    expect(info).to.not.equal(null);
    expect(info!.data.length).to.equal(CONFIG_ACCOUNT_SPACE);
    expect(info!.owner.equals(program.programId)).to.equal(true);
  });

  it("type cosplay: rejects Config-sized account with wrong discriminator", async () => {
    const seed = new BN(601);
    const [mintX, mintY] = await createPairMints();
    await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 2_000_000n);

    // Fake account: same size as Config, owned by the program, zeroed discriminator.
    const fake = Keypair.generate();
    const lamports = await connection.getMinimumBalanceForRentExemption(CONFIG_ACCOUNT_SPACE);
    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: fake.publicKey,
        lamports,
        space: CONFIG_ACCOUNT_SPACE,
        programId: program.programId,
      })
    );
    await sendAndConfirmTransaction(connection, tx, [wallet.payer, fake]);

    try {
      await deposit(
        seed,
        mintX,
        mintY,
        new BN(1_000_000),
        new BN(1_000_000 - MINIMUM_LIQUIDITY),
        { config: fake.publicKey }
      );
      expect.fail("expected type cosplay to fail");
    } catch (err: unknown) {
      expect(errorBlob(err)).to.match(
        /AccountDiscriminator|AccountDidNotDeserialize|ConstraintSeeds|custom program error/i
      );
    }
  });

  it("arbitrary CPI: rejects non-Token program as token_program", async () => {
    const seed = new BN(602);
    const [mintX, mintY] = await createPairMints();
    await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 2_000_000n);

    try {
      await deposit(
        seed,
        mintX,
        mintY,
        new BN(1_000_000),
        new BN(1_000_000 - MINIMUM_LIQUIDITY),
        { tokenProgram: SystemProgram.programId }
      );
      expect.fail("expected arbitrary CPI to fail");
    } catch (err: unknown) {
      expect(errorBlob(err)).to.match(
        /InvalidProgramId|InterfaceAccount|AccountOwnedByWrongProgram|custom program error|Constraint/i
      );
    }
  });

  it("duplicate accounts: rejects vault_x == vault_y", async () => {
    const seed = new BN(603);
    const [mintX, mintY] = await createPairMints();
    const { config, vaultX } = await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 2_000_000n);

    try {
      await deposit(
        seed,
        mintX,
        mintY,
        new BN(1_000_000),
        new BN(1_000_000 - MINIMUM_LIQUIDITY),
        { vaultY: vaultX }
      );
      expect.fail("expected IdenticalVaults");
    } catch (err: unknown) {
      // Constraint / IdenticalVaults / mint mismatch on associated_token
      expect(errorBlob(err)).to.match(
        /IdenticalVaults|ConstraintTokenMint|ConstraintAssociated|6000|0x1770|custom program error/i
      );
    }

    // Sanity: config still distinct from a real vaultY
    const vaultY = getAssociatedTokenAddressSync(mintY, config, true, TOKEN_PROGRAM_ID);
    expect(vaultX.equals(vaultY)).to.equal(false);
  });

  it("missing signer: rejects when declared user did not sign", async () => {
    const seed = new BN(604);
    const [mintX, mintY] = await createPairMints();
    await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 2_000_000n);

    const impostor = Keypair.generate();
    const { config, mintLp } = derivePoolPdas(seed);
    const vaultX = getAssociatedTokenAddressSync(mintX, config, true, TOKEN_PROGRAM_ID);
    const vaultY = getAssociatedTokenAddressSync(mintY, config, true, TOKEN_PROGRAM_ID);
    // Impostor's ATAs won't exist / won't match wallet balances; force user=impostor without signature
    const userX = getAssociatedTokenAddressSync(mintX, wallet.publicKey, false, TOKEN_PROGRAM_ID);
    const userY = getAssociatedTokenAddressSync(mintY, wallet.publicKey, false, TOKEN_PROGRAM_ID);
    const userLp = getAssociatedTokenAddressSync(mintLp, impostor.publicKey, false, TOKEN_PROGRAM_ID);
    const lockLp = getAssociatedTokenAddressSync(mintLp, config, true, TOKEN_PROGRAM_ID);

    try {
      await program.methods
        .deposit(new BN(1_000_000), new BN(1_000_000), new BN(1))
        .accountsPartial({
          user: impostor.publicKey,
          mintX,
          mintY,
          mintLp,
          config,
          vaultX,
          vaultY,
          userX,
          userY,
          userLp,
          lockLp,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([]) // only provider wallet signs — impostor does not
        .rpc();
      expect.fail("expected missing signer failure");
    } catch (err: unknown) {
      expect(errorBlob(err)).to.match(
        /Signature verification|unknown signer|Missing signature|ConstraintRaw|AccountNotSigner|custom program error|Simulation failed/i
      );
    }
  });

  it("first-deposit inflation: rejects 1-token bootstrap that would enable share manipulation", async () => {
    const seed = new BN(605);
    const [mintX, mintY] = await createPairMints();
    await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 10_000n);

    // sqrt(1*1)=1 <= MINIMUM_LIQUIDITY → rejected (cannot seed an attackable pool)
    try {
      await deposit(seed, mintX, mintY, new BN(1), new BN(0));
      expect.fail("expected InsufficientLiquidity");
    } catch (err: unknown) {
      expect(errorBlob(err)).to.match(/InsufficientLiquidity|6005|0x1775/i);
    }

    // Minimal viable first deposit still locks MINIMUM_LIQUIDITY
    const minViable = 1_001; // sqrt(1001*1001)=1001 > 1000
    await fundUser(mintX, mintY, BigInt(minViable * 2));
    await deposit(
      seed,
      mintX,
      mintY,
      new BN(minViable),
      new BN(minViable - MINIMUM_LIQUIDITY)
    );
    const { config, mintLp } = derivePoolPdas(seed);
    const lockLp = getAssociatedTokenAddressSync(mintLp, config, true, TOKEN_PROGRAM_ID);
    const lock = await getAccount(connection, lockLp);
    expect(Number(lock.amount)).to.equal(MINIMUM_LIQUIDITY);
  });

  it("PDA bump: rejects deposit against another pool's Config (wrong seeds/bump)", async () => {
    const seedA = new BN(606);
    const seedB = new BN(607);
    const [mintX, mintY] = await createPairMints();
    const poolA = await initializePool(seedA, mintX, mintY);
    const poolB = await initializePool(seedB, mintX, mintY);
    await fundUser(mintX, mintY, 2_000_000n);

    const { mintLp } = derivePoolPdas(seedA);
    const userX = getAssociatedTokenAddressSync(
      mintX,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const userY = getAssociatedTokenAddressSync(
      mintY,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const userLp = getAssociatedTokenAddressSync(
      mintLp,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const lockLp = getAssociatedTokenAddressSync(mintLp, poolA.config, true, TOKEN_PROGRAM_ID);

    try {
      // Vaults/mint_lp belong to pool A, but config is pool B (different seed + bump).
      await program.methods
        .deposit(new BN(1_000_000), new BN(1_000_000), new BN(1))
        .accountsPartial({
          user: wallet.publicKey,
          mintX,
          mintY,
          mintLp,
          config: poolB.config,
          vaultX: poolA.vaultX,
          vaultY: poolA.vaultY,
          userX,
          userY,
          userLp,
          lockLp,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      expect.fail("expected ConstraintSeeds / has_one failure");
    } catch (err: unknown) {
      expect(errorBlob(err)).to.match(
        /ConstraintSeeds|ConstraintHasOne|ConstraintAssociated|A seeds constraint|custom program error|Simulation failed/i
      );
    }
  });

  it("wrong authority: cannot withdraw LP owned by another wallet", async () => {
    const seed = new BN(608);
    const [mintX, mintY] = await createPairMints();
    await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 2_000_000n);
    await deposit(
      seed,
      mintX,
      mintY,
      new BN(1_000_000),
      new BN(1_000_000 - MINIMUM_LIQUIDITY)
    );

    const thief = Keypair.generate();
    // Airdrop for rent if needed — withdraw doesn't create accounts, but thief must sign
    const sig = await connection.requestAirdrop(thief.publicKey, 1_000_000_000);
    await connection.confirmTransaction(sig, "confirmed");

    const { config, mintLp } = derivePoolPdas(seed);
    const vaultX = getAssociatedTokenAddressSync(mintX, config, true, TOKEN_PROGRAM_ID);
    const vaultY = getAssociatedTokenAddressSync(mintY, config, true, TOKEN_PROGRAM_ID);
    // Victim's LP ATA passed while thief is the signer → ConstraintTokenOwner / authority mismatch
    const victimLp = getAssociatedTokenAddressSync(
      mintLp,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const thiefX = await createAssociatedTokenAccountIdempotent(
      connection,
      thief,
      mintX,
      thief.publicKey,
      undefined,
      TOKEN_PROGRAM_ID
    );
    const thiefY = await createAssociatedTokenAccountIdempotent(
      connection,
      thief,
      mintY,
      thief.publicKey,
      undefined,
      TOKEN_PROGRAM_ID
    );

    try {
      await program.methods
        .withdraw(new BN(100_000), new BN(0), new BN(0))
        .accountsPartial({
          user: thief.publicKey,
          mintX,
          mintY,
          mintLp,
          config,
          vaultX,
          vaultY,
          userX: thiefX,
          userY: thiefY,
          userLp: victimLp,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([thief])
        .rpc();
      expect.fail("expected wrong-authority withdraw to fail");
    } catch (err: unknown) {
      expect(errorBlob(err)).to.match(
        /ConstraintTokenOwner|ConstraintAssociated|owner|custom program error|Simulation failed/i
      );
    }
  });
});
