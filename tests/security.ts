import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import {
  createMint,
  createAssociatedTokenAccountIdempotent,
  mintTo,
  getAccount,
  transferChecked,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ExtensionType,
  getMintLen,
  createInitializeMintInstruction,
  createInitializeTransferFeeConfigInstruction,
  createInitializePermanentDelegateInstruction,
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
import { AssertionError, expect } from "chai";
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
    // An `expect.fail` thrown inside `try` must not be matched (and swallowed) by the `catch`.
    if (err instanceof AssertionError) throw err;
    const anchorErr = err as {
      message?: string;
      logs?: string[];
      error?: { errorCode?: { code?: string; number?: number } };
    };
    const logs = (anchorErr.logs ?? []).join("\n");
    const code = anchorErr.error?.errorCode?.code ?? "";
    return `${anchorErr.message ?? ""}\n${logs}\n${code}`;
  }

  /** Awaits `action` and asserts it rejects with an error matching `pattern`. */
  async function expectError(
    action: () => Promise<unknown>,
    pattern: RegExp
  ): Promise<void> {
    let caught: unknown = undefined;
    let rejected = false;
    try {
      await action();
    } catch (err: unknown) {
      rejected = true;
      caught = err;
    }
    if (!rejected) expect.fail(`expected rejection matching ${pattern}`);
    expect(errorBlob(caught)).to.match(pattern);
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

  type Token2022Extension = "none" | "transferFee" | "permanentDelegate";

  async function createToken2022Mint(
    extension: Token2022Extension
  ): Promise<PublicKey> {
    const mint = Keypair.generate();
    const extensions =
      extension === "transferFee"
        ? [ExtensionType.TransferFeeConfig]
        : extension === "permanentDelegate"
        ? [ExtensionType.PermanentDelegate]
        : [];
    const space = getMintLen(extensions);
    const lamports = await connection.getMinimumBalanceForRentExemption(space);

    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: mint.publicKey,
        space,
        lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      })
    );
    if (extension === "transferFee") {
      tx.add(
        createInitializeTransferFeeConfigInstruction(
          mint.publicKey,
          wallet.publicKey,
          wallet.publicKey,
          100, // 1% fee on every transfer
          BigInt(1_000_000_000),
          TOKEN_2022_PROGRAM_ID
        )
      );
    } else if (extension === "permanentDelegate") {
      tx.add(
        createInitializePermanentDelegateInstruction(
          mint.publicKey,
          wallet.publicKey,
          TOKEN_2022_PROGRAM_ID
        )
      );
    }
    tx.add(
      createInitializeMintInstruction(
        mint.publicKey,
        6,
        wallet.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID
      )
    );
    await sendAndConfirmTransaction(connection, tx, [wallet.payer, mint]);
    return mint.publicKey;
  }

  async function initializePool(
    seed: BN,
    mintX: PublicKey,
    mintY: PublicKey,
    fee = 30,
    tokenProgram: PublicKey = TOKEN_PROGRAM_ID,
    authority: PublicKey | null = null
  ) {
    const { config, mintLp } = derivePoolPdas(seed);
    const vaultX = getAssociatedTokenAddressSync(
      mintX,
      config,
      true,
      tokenProgram
    );
    const vaultY = getAssociatedTokenAddressSync(
      mintY,
      config,
      true,
      tokenProgram
    );

    await program.methods
      .initialize(seed, fee, authority)
      .accountsPartial({
        initializer: wallet.publicKey,
        mintX,
        mintY,
        config,
        mintLp,
        vaultX,
        vaultY,
        tokenProgram,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    return { config, mintLp, vaultX, vaultY };
  }

  async function fundUser(
    mintX: PublicKey,
    mintY: PublicKey,
    amount: bigint,
    tokenProgram: PublicKey = TOKEN_PROGRAM_ID
  ) {
    const userX = await createAssociatedTokenAccountIdempotent(
      connection,
      wallet.payer,
      mintX,
      wallet.publicKey,
      undefined,
      tokenProgram
    );
    const userY = await createAssociatedTokenAccountIdempotent(
      connection,
      wallet.payer,
      mintY,
      wallet.publicKey,
      undefined,
      tokenProgram
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
      tokenProgram
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
      tokenProgram
    );
    return { userX, userY };
  }

  async function deposit(
    seed: BN,
    mintX: PublicKey,
    mintY: PublicKey,
    amount: BN | { x: BN; y: BN },
    minLp: BN,
    overrides: Record<string, PublicKey> = {},
    tokenProgram: PublicKey = TOKEN_PROGRAM_ID
  ) {
    const { config, mintLp } = derivePoolPdas(seed);
    const vaultX = getAssociatedTokenAddressSync(
      mintX,
      config,
      true,
      tokenProgram
    );
    const vaultY = getAssociatedTokenAddressSync(
      mintY,
      config,
      true,
      tokenProgram
    );
    const userX = getAssociatedTokenAddressSync(
      mintX,
      wallet.publicKey,
      false,
      tokenProgram
    );
    const userY = getAssociatedTokenAddressSync(
      mintY,
      wallet.publicKey,
      false,
      tokenProgram
    );
    const userLp = getAssociatedTokenAddressSync(
      mintLp,
      wallet.publicKey,
      false,
      tokenProgram
    );
    const lockLp = getAssociatedTokenAddressSync(
      mintLp,
      config,
      true,
      tokenProgram
    );

    return program.methods
      .deposit(
        BN.isBN(amount) ? amount : amount.x,
        BN.isBN(amount) ? amount : amount.y,
        minLp
      )
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
        tokenProgram,
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
    const lamports = await connection.getMinimumBalanceForRentExemption(
      CONFIG_ACCOUNT_SPACE
    );
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

    await expectError(
      () =>
        deposit(
          seed,
          mintX,
          mintY,
          new BN(1_000_000),
          new BN(1_000_000 - MINIMUM_LIQUIDITY),
          { config: fake.publicKey }
        ),
      /AccountDiscriminator|AccountDidNotDeserialize|ConstraintSeeds|custom program error/i
    );
  });

  it("arbitrary CPI: rejects non-Token program as token_program", async () => {
    const seed = new BN(602);
    const [mintX, mintY] = await createPairMints();
    await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 2_000_000n);

    await expectError(
      () =>
        deposit(
          seed,
          mintX,
          mintY,
          new BN(1_000_000),
          new BN(1_000_000 - MINIMUM_LIQUIDITY),
          { tokenProgram: SystemProgram.programId }
        ),
      /InvalidProgramId|InterfaceAccount|AccountOwnedByWrongProgram|custom program error|Constraint/i
    );
  });

  it("duplicate accounts: rejects vault_x == vault_y", async () => {
    const seed = new BN(603);
    const [mintX, mintY] = await createPairMints();
    const { config, vaultX } = await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 2_000_000n);

    // Constraint / IdenticalVaults / mint mismatch on associated_token
    await expectError(
      () =>
        deposit(
          seed,
          mintX,
          mintY,
          new BN(1_000_000),
          new BN(1_000_000 - MINIMUM_LIQUIDITY),
          { vaultY: vaultX }
        ),
      /IdenticalVaults|ConstraintTokenMint|ConstraintAssociated|6000|0x1770|custom program error/i
    );

    // Sanity: config still distinct from a real vaultY
    const vaultY = getAssociatedTokenAddressSync(
      mintY,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
    expect(vaultX.equals(vaultY)).to.equal(false);
  });

  it("missing signer: rejects when declared user did not sign", async () => {
    const seed = new BN(604);
    const [mintX, mintY] = await createPairMints();
    await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 2_000_000n);

    const impostor = Keypair.generate();
    const { config, mintLp } = derivePoolPdas(seed);
    const vaultX = getAssociatedTokenAddressSync(
      mintX,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
    const vaultY = getAssociatedTokenAddressSync(
      mintY,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
    // Impostor's ATAs won't exist / won't match wallet balances; force user=impostor without signature
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
      impostor.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const lockLp = getAssociatedTokenAddressSync(
      mintLp,
      config,
      true,
      TOKEN_PROGRAM_ID
    );

    await expectError(
      () =>
        program.methods
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
          .rpc(),
      /Signature verification|unknown signer|Missing signature|ConstraintRaw|AccountNotSigner|custom program error|Simulation failed/i
    );
  });

  it("first-deposit inflation: rejects 1-token bootstrap that would enable share manipulation", async () => {
    const seed = new BN(605);
    const [mintX, mintY] = await createPairMints();
    await initializePool(seed, mintX, mintY);
    await fundUser(mintX, mintY, 10_000n);

    // sqrt(1*1)=1 <= MINIMUM_LIQUIDITY → rejected (cannot seed an attackable pool)
    await expectError(
      () => deposit(seed, mintX, mintY, new BN(1), new BN(0)),
      /InsufficientLiquidity|6005|0x1775/i
    );

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
    const lockLp = getAssociatedTokenAddressSync(
      mintLp,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
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
    const lockLp = getAssociatedTokenAddressSync(
      mintLp,
      poolA.config,
      true,
      TOKEN_PROGRAM_ID
    );

    // Vaults/mint_lp belong to pool A, but config is pool B (different seed + bump).
    await expectError(
      () =>
        program.methods
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
          .rpc(),
      /ConstraintSeeds|ConstraintHasOne|ConstraintAssociated|A seeds constraint|custom program error|Simulation failed/i
    );
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
    const vaultX = getAssociatedTokenAddressSync(
      mintX,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
    const vaultY = getAssociatedTokenAddressSync(
      mintY,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
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

    await expectError(
      () =>
        program.methods
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
          .rpc(),
      /ConstraintTokenOwner|ConstraintAssociated|owner|custom program error|Simulation failed/i
    );
  });

  it("donation griefing: 1-unit transfer into an empty vault does not brick the first deposit", async () => {
    const seed = new BN(609);
    const [mintX, mintY] = await createPairMints();
    const { config, mintLp, vaultX } = await initializePool(seed, mintX, mintY);
    const { userX } = await fundUser(mintX, mintY, 2_000_000n);

    // Attacker donates directly to the predictable vault ATA before any LP exists.
    await transferChecked(
      connection,
      wallet.payer,
      userX,
      mintX,
      vaultX,
      wallet.payer,
      1n,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );

    await deposit(
      seed,
      mintX,
      mintY,
      new BN(1_000_000),
      new BN(1_000_000 - MINIMUM_LIQUIDITY)
    );

    const userLp = getAssociatedTokenAddressSync(
      mintLp,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const lockLp = getAssociatedTokenAddressSync(
      mintLp,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
    expect(Number((await getAccount(connection, userLp)).amount)).to.equal(
      1_000_000 - MINIMUM_LIQUIDITY
    );
    expect(Number((await getAccount(connection, lockLp)).amount)).to.equal(
      MINIMUM_LIQUIDITY
    );
    expect(Number((await getAccount(connection, vaultX)).amount)).to.equal(
      1_000_001
    );
  });

  it("token-2022: rejects mints with TransferFeeConfig (vault would receive less than credited)", async () => {
    const seed = new BN(610);
    const mintX = await createToken2022Mint("transferFee");
    const mintY = await createToken2022Mint("none");

    await expectError(
      () => initializePool(seed, mintX, mintY, 30, TOKEN_2022_PROGRAM_ID),
      /UnsupportedMintExtension/
    );
  });

  it("token-2022: rejects mints with PermanentDelegate (vault could be drained by the delegate)", async () => {
    const seed = new BN(611);
    const mintX = await createToken2022Mint("none");
    const mintY = await createToken2022Mint("permanentDelegate");

    await expectError(
      () => initializePool(seed, mintX, mintY, 30, TOKEN_2022_PROGRAM_ID),
      /UnsupportedMintExtension/
    );
  });

  it("token-2022: plain mints without extensions initialize and accept deposits", async () => {
    const seed = new BN(612);
    const mintX = await createToken2022Mint("none");
    const mintY = await createToken2022Mint("none");

    const { config, mintLp, vaultX, vaultY } = await initializePool(
      seed,
      mintX,
      mintY,
      30,
      TOKEN_2022_PROGRAM_ID
    );
    const vaultInfo = await connection.getAccountInfo(vaultX);
    expect(vaultInfo!.owner.equals(TOKEN_2022_PROGRAM_ID)).to.equal(true);

    await fundUser(mintX, mintY, 2_000_000n, TOKEN_2022_PROGRAM_ID);
    await deposit(
      seed,
      mintX,
      mintY,
      new BN(1_000_000),
      new BN(1_000_000 - MINIMUM_LIQUIDITY),
      {},
      TOKEN_2022_PROGRAM_ID
    );

    const lockLp = getAssociatedTokenAddressSync(
      mintLp,
      config,
      true,
      TOKEN_2022_PROGRAM_ID
    );
    const lock = await getAccount(
      connection,
      lockLp,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    expect(Number(lock.amount)).to.equal(MINIMUM_LIQUIDITY);
    const vy = await getAccount(
      connection,
      vaultY,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    expect(Number(vy.amount)).to.equal(1_000_000);
  });

  it("unbalanced deposit: only the pool ratio is charged, the excess stays with the user", async () => {
    const seed = new BN(613);
    const [mintX, mintY] = await createPairMints();
    const { mintLp, vaultX, vaultY } = await initializePool(seed, mintX, mintY);
    const { userX, userY } = await fundUser(mintX, mintY, 2_000_000n);
    await deposit(
      seed,
      mintX,
      mintY,
      new BN(1_000_000),
      new BN(1_000_000 - MINIMUM_LIQUIDITY)
    );

    // Pool is 1:1; offering 100k X + 500k Y must only take 100k Y.
    await deposit(
      seed,
      mintX,
      mintY,
      { x: new BN(100_000), y: new BN(500_000) },
      new BN(100_000)
    );

    expect(Number((await getAccount(connection, userX)).amount)).to.equal(
      900_000
    );
    expect(Number((await getAccount(connection, userY)).amount)).to.equal(
      900_000
    );
    expect(Number((await getAccount(connection, vaultX)).amount)).to.equal(
      1_100_000
    );
    expect(Number((await getAccount(connection, vaultY)).amount)).to.equal(
      1_100_000
    );
    const userLp = getAssociatedTokenAddressSync(
      mintLp,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    expect(Number((await getAccount(connection, userLp)).amount)).to.equal(
      1_000_000 - MINIMUM_LIQUIDITY + 100_000
    );
  });

  it("freeze authority: rejects mints that could freeze the pool vaults", async () => {
    const seed = new BN(614);
    const freezable = await createMint(
      connection,
      wallet.payer,
      wallet.publicKey,
      wallet.publicKey,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    const [, mintY] = await createPairMints();

    await expectError(
      () => initializePool(seed, freezable, mintY),
      /MintHasFreezeAuthority/
    );
  });

  async function setLocked(
    config: PublicKey,
    locked: boolean,
    signer: Keypair = wallet.payer
  ) {
    return program.methods
      .setLocked(locked)
      .accountsPartial({ authority: signer.publicKey, config })
      .signers(signer === wallet.payer ? [] : [signer])
      .rpc();
  }

  async function setAuthority(
    config: PublicKey,
    newAuthority: PublicKey | null,
    signer: Keypair = wallet.payer
  ) {
    return program.methods
      .setAuthority(newAuthority)
      .accountsPartial({ authority: signer.publicKey, config })
      .signers(signer === wallet.payer ? [] : [signer])
      .rpc();
  }

  async function fundedKeypair(): Promise<Keypair> {
    const kp = Keypair.generate();
    const sig = await connection.requestAirdrop(kp.publicKey, 1_000_000_000);
    await connection.confirmTransaction(sig, "confirmed");
    return kp;
  }

  it("admin: authority pauses deposits and swaps while withdrawals stay open", async () => {
    const seed = new BN(615);
    const [mintX, mintY] = await createPairMints();
    const { config, mintLp, vaultX, vaultY } = await initializePool(
      seed,
      mintX,
      mintY,
      30,
      TOKEN_PROGRAM_ID,
      wallet.publicKey
    );
    const { userX, userY } = await fundUser(mintX, mintY, 3_000_000n);
    await deposit(
      seed,
      mintX,
      mintY,
      new BN(1_000_000),
      new BN(1_000_000 - MINIMUM_LIQUIDITY)
    );

    await setLocked(config, true);
    expect((await program.account.config.fetch(config)).locked).to.equal(true);

    await expectError(
      () => deposit(seed, mintX, mintY, new BN(100_000), new BN(0)),
      /PoolLocked/
    );

    await expectError(
      () =>
        program.methods
          .swap(true, new BN(10_000), new BN(0))
          .accountsPartial({
            user: wallet.publicKey,
            mintX,
            mintY,
            config,
            vaultX,
            vaultY,
            userX,
            userY,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc(),
      /PoolLocked/
    );

    // LPs can always exit: a pause must never trap funds.
    const userLp = getAssociatedTokenAddressSync(
      mintLp,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    await program.methods
      .withdraw(new BN(100_000), new BN(1), new BN(1))
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
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    expect(Number((await getAccount(connection, userLp)).amount)).to.equal(
      1_000_000 - MINIMUM_LIQUIDITY - 100_000
    );

    await setLocked(config, false);
    await deposit(seed, mintX, mintY, new BN(100_000), new BN(1));
  });

  it("admin: rejects set_locked from a non-authority signer or on a pool without admin", async () => {
    const [mintX, mintY] = await createPairMints();
    const withAdmin = await initializePool(
      new BN(616),
      mintX,
      mintY,
      30,
      TOKEN_PROGRAM_ID,
      wallet.publicKey
    );
    const stranger = await fundedKeypair();

    await expectError(
      () => setLocked(withAdmin.config, true, stranger),
      /Unauthorized/
    );

    const noAdmin = await initializePool(new BN(617), mintX, mintY);
    await expectError(() => setLocked(noAdmin.config, true), /Unauthorized/);
    expect(
      (await program.account.config.fetch(noAdmin.config)).locked
    ).to.equal(false);
  });

  it("admin: authority can be transferred and renounced", async () => {
    const [mintX, mintY] = await createPairMints();
    const { config } = await initializePool(
      new BN(618),
      mintX,
      mintY,
      30,
      TOKEN_PROGRAM_ID,
      wallet.publicKey
    );
    const nextAdmin = await fundedKeypair();

    await setAuthority(config, nextAdmin.publicKey);
    expect(
      (await program.account.config.fetch(config)).authority!.equals(
        nextAdmin.publicKey
      )
    ).to.equal(true);

    await expectError(() => setLocked(config, true), /Unauthorized/);

    await setLocked(config, true, nextAdmin);
    await setLocked(config, false, nextAdmin);

    await setAuthority(config, null, nextAdmin);
    expect((await program.account.config.fetch(config)).authority).to.equal(
      null
    );
    await expectError(() => setLocked(config, true, nextAdmin), /Unauthorized/);
  });

  it("withdraw: rejects burning more LP than the signer holds even when the pool could pay", async () => {
    const seed = new BN(619);
    const [mintX, mintY] = await createPairMints();
    const { config, mintLp, vaultX, vaultY } = await initializePool(
      seed,
      mintX,
      mintY
    );
    await fundUser(mintX, mintY, 1_000_000n);
    await deposit(
      seed,
      mintX,
      mintY,
      new BN(1_000_000),
      new BN(1_000_000 - MINIMUM_LIQUIDITY)
    );

    const lp2 = await fundedKeypair();
    const atas = await Promise.all(
      [mintX, mintY].map((mint) =>
        createAssociatedTokenAccountIdempotent(
          connection,
          wallet.payer,
          mint,
          lp2.publicKey,
          undefined,
          TOKEN_PROGRAM_ID
        )
      )
    );
    for (const [i, mint] of [mintX, mintY].entries()) {
      await mintTo(
        connection,
        wallet.payer,
        mint,
        atas[i],
        wallet.payer,
        100_000n,
        undefined,
        undefined,
        TOKEN_PROGRAM_ID
      );
    }
    const [lp2X, lp2Y] = atas;
    const lp2Lp = getAssociatedTokenAddressSync(
      mintLp,
      lp2.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const lockLp = getAssociatedTokenAddressSync(
      mintLp,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
    await program.methods
      .deposit(new BN(100_000), new BN(100_000), new BN(100_000))
      .accountsPartial({
        user: lp2.publicKey,
        mintX,
        mintY,
        mintLp,
        config,
        vaultX,
        vaultY,
        userX: lp2X,
        userY: lp2Y,
        userLp: lp2Lp,
        lockLp,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([lp2])
      .rpc();

    // Supply is 1.1M, so burning 200k keeps the lock: only the holder-balance check can stop it.
    await expectError(
      () =>
        program.methods
          .withdraw(new BN(200_000), new BN(0), new BN(0))
          .accountsPartial({
            user: lp2.publicKey,
            mintX,
            mintY,
            mintLp,
            config,
            vaultX,
            vaultY,
            userX: lp2X,
            userY: lp2Y,
            userLp: lp2Lp,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([lp2])
          .rpc(),
      /InsufficientLiquidity/
    );
  });

  it("initialize: rejects vault_x == vault_y with IdenticalVaults", async () => {
    const seed = new BN(620);
    const [mintX, mintY] = await createPairMints();
    const { config, mintLp } = derivePoolPdas(seed);
    const vaultX = getAssociatedTokenAddressSync(
      mintX,
      config,
      true,
      TOKEN_PROGRAM_ID
    );

    await expectError(
      () =>
        program.methods
          .initialize(seed, 30, null)
          .accountsPartial({
            initializer: wallet.publicKey,
            mintX,
            mintY,
            config,
            mintLp,
            vaultX,
            vaultY: vaultX,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .rpc(),
      /IdenticalVaults/
    );
  });
});
