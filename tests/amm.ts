import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import {
  createMint,
  createAssociatedTokenAccountIdempotent,
  mintTo,
  getAccount,
  getMint,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { Amm } from "../target/types/amm";

/**
 * Manual Config layout (must stay in sync with `state/config.rs` InitSpace):
 * Option<Pubkey> 33 + Pubkey×3 96 + u64 8 + u16 2 + bool 1 + u8×2 2 = 142
 * Account space = 8 (discriminator) + 142 = 150
 */
const CONFIG_INIT_SPACE = 142;
const CONFIG_ACCOUNT_SPACE = 8 + CONFIG_INIT_SPACE;
const CONFIG_SEED = Buffer.from("config");
const LP_SEED = Buffer.from("lp");
const MINIMUM_LIQUIDITY = 1_000;

describe("amm", () => {
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

  async function initializePool(args: {
    seed: BN;
    fee: number;
    authority: PublicKey | null;
    mintX: PublicKey;
    mintY: PublicKey;
  }) {
    const { config, mintLp } = derivePoolPdas(args.seed);
    const vaultX = getAssociatedTokenAddressSync(
      args.mintX,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
    const vaultY = getAssociatedTokenAddressSync(
      args.mintY,
      config,
      true,
      TOKEN_PROGRAM_ID
    );

    const tx = await program.methods
      .initialize(args.seed, args.fee, args.authority)
      .accountsPartial({
        initializer: wallet.publicKey,
        mintX: args.mintX,
        mintY: args.mintY,
        config,
        mintLp,
        vaultX,
        vaultY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    return { tx, config, mintLp, vaultX, vaultY };
  }

  async function fundUserAtas(
    mintX: PublicKey,
    mintY: PublicKey,
    amountX: bigint,
    amountY: bigint
  ) {
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
    if (amountX > 0n) {
      await mintTo(
        connection,
        wallet.payer,
        mintX,
        userX,
        wallet.payer,
        amountX,
        undefined,
        undefined,
        TOKEN_PROGRAM_ID
      );
    }
    if (amountY > 0n) {
      await mintTo(
        connection,
        wallet.payer,
        mintY,
        userY,
        wallet.payer,
        amountY,
        undefined,
        undefined,
        TOKEN_PROGRAM_ID
      );
    }
    return { userX, userY };
  }

  async function depositLiquidity(args: {
    seed: BN;
    mintX: PublicKey;
    mintY: PublicKey;
    amountX: BN;
    amountY: BN;
    minLp: BN;
  }) {
    const { config, mintLp } = derivePoolPdas(args.seed);
    const vaultX = getAssociatedTokenAddressSync(
      args.mintX,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
    const vaultY = getAssociatedTokenAddressSync(
      args.mintY,
      config,
      true,
      TOKEN_PROGRAM_ID
    );
    const userX = getAssociatedTokenAddressSync(
      args.mintX,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const userY = getAssociatedTokenAddressSync(
      args.mintY,
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
      config,
      true,
      TOKEN_PROGRAM_ID
    );

    const tx = await program.methods
      .deposit(args.amountX, args.amountY, args.minLp)
      .accountsPartial({
        user: wallet.publicKey,
        mintX: args.mintX,
        mintY: args.mintY,
        mintLp,
        config,
        vaultX,
        vaultY,
        userX,
        userY,
        userLp,
        lockLp,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    return { tx, config, mintLp, vaultX, vaultY, userLp, lockLp };
  }

  it("phase1: Config account space is discriminator + InitSpace", () => {
    expect(CONFIG_ACCOUNT_SPACE).to.equal(150);
    expect(CONFIG_INIT_SPACE).to.equal(142);
  });

  describe("phase2: initialize", () => {
    let mintX: PublicKey;
    let mintY: PublicKey;

    before(async () => {
      [mintX, mintY] = await createPairMints();
    });

    it("creates Config, LP mint and vaults with canonical bumps", async () => {
      const seed = new BN(42);
      const fee = 30;
      const authority = wallet.publicKey;

      const { config, mintLp, vaultX, vaultY } = await initializePool({
        seed,
        fee,
        authority,
        mintX,
        mintY,
      });

      const { configBump, lpBump } = derivePoolPdas(seed);
      const configAccount = await program.account.config.fetch(config);

      expect(configAccount.seed.toString()).to.equal(seed.toString());
      expect(configAccount.fee).to.equal(fee);
      expect(configAccount.locked).to.equal(false);
      expect(configAccount.authority!.toBase58()).to.equal(authority.toBase58());
      expect(configAccount.mintX.toBase58()).to.equal(mintX.toBase58());
      expect(configAccount.mintY.toBase58()).to.equal(mintY.toBase58());
      expect(configAccount.mintLp.toBase58()).to.equal(mintLp.toBase58());
      expect(configAccount.configBump).to.equal(configBump);
      expect(configAccount.lpBump).to.equal(lpBump);

      const configInfo = await connection.getAccountInfo(config);
      expect(configInfo).to.not.equal(null);
      expect(configInfo!.data.length).to.equal(CONFIG_ACCOUNT_SPACE);
      expect(vaultX.equals(vaultY)).to.equal(false);

      const vaultXInfo = await connection.getAccountInfo(vaultX);
      const vaultYInfo = await connection.getAccountInfo(vaultY);
      expect(vaultXInfo).to.not.equal(null);
      expect(vaultYInfo).to.not.equal(null);
    });

    it("rejects double initialize for the same seed", async () => {
      const seed = new BN(43);
      const [x, y] = await createPairMints();

      await initializePool({
        seed,
        fee: 25,
        authority: null,
        mintX: x,
        mintY: y,
      });

      try {
        await initializePool({
          seed,
          fee: 25,
          authority: null,
          mintX: x,
          mintY: y,
        });
        expect.fail("expected second initialize to fail");
      } catch (err: unknown) {
        expect(err).to.exist;
      }
    });

    it("rejects identical mint_x and mint_y", async () => {
      const seed = new BN(44);
      try {
        await initializePool({
          seed,
          fee: 30,
          authority: null,
          mintX,
          mintY: mintX,
        });
        expect.fail("expected IdenticalMints error");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/IdenticalMints|6001|0x1771/i);
      }
    });

    it("rejects fee above MAX_FEE_BPS", async () => {
      const seed = new BN(45);
      const [x, y] = await createPairMints();
      try {
        await initializePool({
          seed,
          fee: 10_001,
          authority: null,
          mintX: x,
          mintY: y,
        });
        expect.fail("expected InvalidFee error");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/InvalidFee|6002|0x1772/i);
      }
    });
  });

  describe("phase3: deposit", () => {
    it("first deposit mints LP and locks MINIMUM_LIQUIDITY", async () => {
      const seed = new BN(100);
      const [mintX, mintY] = await createPairMints();
      await initializePool({
        seed,
        fee: 30,
        authority: null,
        mintX,
        mintY,
      });

      const amount = 1_000_000n;
      await fundUserAtas(mintX, mintY, amount, amount);

      const expectedUserLp = Number(amount) - MINIMUM_LIQUIDITY;
      const { mintLp, vaultX, vaultY, userLp, lockLp } = await depositLiquidity({
        seed,
        mintX,
        mintY,
        amountX: new BN(amount.toString()),
        amountY: new BN(amount.toString()),
        minLp: new BN(expectedUserLp),
      });

      const userLpAccount = await getAccount(connection, userLp);
      const lockLpAccount = await getAccount(connection, lockLp);
      const lpMint = await getMint(connection, mintLp);
      const vaultXAccount = await getAccount(connection, vaultX);
      const vaultYAccount = await getAccount(connection, vaultY);

      expect(Number(userLpAccount.amount)).to.equal(expectedUserLp);
      expect(Number(lockLpAccount.amount)).to.equal(MINIMUM_LIQUIDITY);
      expect(Number(lpMint.supply)).to.equal(Number(amount));
      expect(Number(vaultXAccount.amount)).to.equal(Number(amount));
      expect(Number(vaultYAccount.amount)).to.equal(Number(amount));
    });

    it("subsequent deposit mints proportional LP", async () => {
      const seed = new BN(101);
      const [mintX, mintY] = await createPairMints();
      await initializePool({
        seed,
        fee: 30,
        authority: null,
        mintX,
        mintY,
      });

      await fundUserAtas(mintX, mintY, 2_000_000n, 2_000_000n);

      await depositLiquidity({
        seed,
        mintX,
        mintY,
        amountX: new BN(1_000_000),
        amountY: new BN(1_000_000),
        minLp: new BN(1_000_000 - MINIMUM_LIQUIDITY),
      });

      const { userLp, mintLp } = await depositLiquidity({
        seed,
        mintX,
        mintY,
        amountX: new BN(500_000),
        amountY: new BN(500_000),
        minLp: new BN(500_000),
      });

      const userLpAccount = await getAccount(connection, userLp);
      const lpMint = await getMint(connection, mintLp);
      // first: 999_000 to user + 1000 lock; second: +500_000 → user total 1_499_000
      expect(Number(userLpAccount.amount)).to.equal(1_499_000);
      expect(Number(lpMint.supply)).to.equal(1_500_000);
    });

    it("rejects zero amounts", async () => {
      const seed = new BN(102);
      const [mintX, mintY] = await createPairMints();
      await initializePool({
        seed,
        fee: 30,
        authority: null,
        mintX,
        mintY,
      });
      await fundUserAtas(mintX, mintY, 1_000_000n, 1_000_000n);

      try {
        await depositLiquidity({
          seed,
          mintX,
          mintY,
          amountX: new BN(0),
          amountY: new BN(1_000_000),
          minLp: new BN(0),
        });
        expect.fail("expected InvalidAmount");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/InvalidAmount|6008|0x1778/i);
      }
    });

    it("rejects slippage when min_lp is too high", async () => {
      const seed = new BN(103);
      const [mintX, mintY] = await createPairMints();
      await initializePool({
        seed,
        fee: 30,
        authority: null,
        mintX,
        mintY,
      });
      await fundUserAtas(mintX, mintY, 1_000_000n, 1_000_000n);

      try {
        await depositLiquidity({
          seed,
          mintX,
          mintY,
          amountX: new BN(1_000_000),
          amountY: new BN(1_000_000),
          minLp: new BN(2_000_000),
        });
        expect.fail("expected SlippageExceeded");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/SlippageExceeded|6003|0x1773/i);
      }
    });

    it("rejects first deposit below MINIMUM_LIQUIDITY", async () => {
      const seed = new BN(104);
      const [mintX, mintY] = await createPairMints();
      await initializePool({
        seed,
        fee: 30,
        authority: null,
        mintX,
        mintY,
      });
      await fundUserAtas(mintX, mintY, 100n, 100n);

      try {
        await depositLiquidity({
          seed,
          mintX,
          mintY,
          amountX: new BN(100),
          amountY: new BN(100),
          minLp: new BN(0),
        });
        expect.fail("expected InsufficientLiquidity");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/InsufficientLiquidity|6005|0x1775/i);
      }
    });
  });

  describe("phase4: withdraw", () => {
    async function withdrawLiquidity(args: {
      seed: BN;
      mintX: PublicKey;
      mintY: PublicKey;
      lpAmount: BN;
      minX: BN;
      minY: BN;
    }) {
      const { config, mintLp } = derivePoolPdas(args.seed);
      const vaultX = getAssociatedTokenAddressSync(
        args.mintX,
        config,
        true,
        TOKEN_PROGRAM_ID
      );
      const vaultY = getAssociatedTokenAddressSync(
        args.mintY,
        config,
        true,
        TOKEN_PROGRAM_ID
      );
      const userX = getAssociatedTokenAddressSync(
        args.mintX,
        wallet.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );
      const userY = getAssociatedTokenAddressSync(
        args.mintY,
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

      const tx = await program.methods
        .withdraw(args.lpAmount, args.minX, args.minY)
        .accountsPartial({
          user: wallet.publicKey,
          mintX: args.mintX,
          mintY: args.mintY,
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

      return { tx, config, mintLp, vaultX, vaultY, userX, userY, userLp };
    }

    async function setupPoolWithLiquidity(seed: BN, depositAmount = 1_000_000n) {
      const [mintX, mintY] = await createPairMints();
      await initializePool({
        seed,
        fee: 30,
        authority: null,
        mintX,
        mintY,
      });
      await fundUserAtas(mintX, mintY, depositAmount * 2n, depositAmount * 2n);
      const deposited = await depositLiquidity({
        seed,
        mintX,
        mintY,
        amountX: new BN(depositAmount.toString()),
        amountY: new BN(depositAmount.toString()),
        minLp: new BN(Number(depositAmount) - MINIMUM_LIQUIDITY),
      });
      return { mintX, mintY, ...deposited, depositAmount };
    }

    it("burns LP and returns proportional X/Y", async () => {
      const seed = new BN(200);
      const { mintX, mintY, userLp, vaultX, vaultY, depositAmount } =
        await setupPoolWithLiquidity(seed);

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

      const burnLp = 500_000;
      const userXBefore = await getAccount(connection, userX);
      const userYBefore = await getAccount(connection, userY);

      await withdrawLiquidity({
        seed,
        mintX,
        mintY,
        lpAmount: new BN(burnLp),
        minX: new BN(burnLp),
        minY: new BN(burnLp),
      });

      const userLpAfter = await getAccount(connection, userLp);
      const vaultXAfter = await getAccount(connection, vaultX);
      const vaultYAfter = await getAccount(connection, vaultY);
      const userXAfter = await getAccount(connection, userX);
      const userYAfter = await getAccount(connection, userY);
      const lpMint = await getMint(connection, derivePoolPdas(seed).mintLp);

      expect(Number(userLpAfter.amount)).to.equal(
        Number(depositAmount) - MINIMUM_LIQUIDITY - burnLp
      );
      expect(Number(vaultXAfter.amount)).to.equal(Number(depositAmount) - burnLp);
      expect(Number(vaultYAfter.amount)).to.equal(Number(depositAmount) - burnLp);
      expect(Number(userXAfter.amount - userXBefore.amount)).to.equal(burnLp);
      expect(Number(userYAfter.amount - userYBefore.amount)).to.equal(burnLp);
      expect(Number(lpMint.supply)).to.equal(Number(depositAmount) - burnLp);
    });

    it("rejects slippage when min_x/min_y are too high", async () => {
      const seed = new BN(201);
      const { mintX, mintY } = await setupPoolWithLiquidity(seed);

      try {
        await withdrawLiquidity({
          seed,
          mintX,
          mintY,
          lpAmount: new BN(100_000),
          minX: new BN(1_000_000),
          minY: new BN(0),
        });
        expect.fail("expected SlippageExceeded");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/SlippageExceeded|6003|0x1773/i);
      }
    });

    it("rejects withdrawing more LP than the user holds", async () => {
      const seed = new BN(202);
      const { mintX, mintY } = await setupPoolWithLiquidity(seed);

      try {
        await withdrawLiquidity({
          seed,
          mintX,
          mintY,
          lpAmount: new BN(2_000_000),
          minX: new BN(0),
          minY: new BN(0),
        });
        expect.fail("expected InsufficientLiquidity");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/InsufficientLiquidity|6005|0x1775/i);
      }
    });

    it("rejects burning that would drop supply below MINIMUM_LIQUIDITY", async () => {
      const seed = new BN(203);
      const { mintX, mintY } = await setupPoolWithLiquidity(seed);
      // User holds 999_000; burning all would leave supply = 1000 (OK).
      // Burning 999_001 is more than user has — use a second depositor scenario:
      // After first deposit supply=1_000_000. Burn 999_001 from a user who somehow
      // had that much — user only has 999_000. So burn exactly 999_000 is OK.
      // To violate remaining >= MINIMUM, need burn > total - MINIMUM = 999_000.
      // User only has 999_000, so burn 999_000 leaves exactly MINIMUM — allowed.
      // Add second deposit so user has more LP, then try to burn past the lock.
      await depositLiquidity({
        seed,
        mintX,
        mintY,
        amountX: new BN(1_000_000),
        amountY: new BN(1_000_000),
        minLp: new BN(1_000_000),
      });
      // supply now 2_000_000; user LP = 999_000 + 1_000_000 = 1_999_000
      // Max burn allowed = 2_000_000 - 1_000 = 1_999_000 (exactly leaves lock)
      // Try burn 1_999_001 → should fail
      try {
        await withdrawLiquidity({
          seed,
          mintX,
          mintY,
          lpAmount: new BN(1_999_001),
          minX: new BN(0),
          minY: new BN(0),
        });
        expect.fail("expected InsufficientLiquidity");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/InsufficientLiquidity|6005|0x1775/i);
      }
    });

    it("allows withdrawing all user LP leaving locked MINIMUM_LIQUIDITY", async () => {
      const seed = new BN(204);
      const { mintX, mintY, userLp, depositAmount } = await setupPoolWithLiquidity(seed);
      const userLpAmount = Number(depositAmount) - MINIMUM_LIQUIDITY;

      await withdrawLiquidity({
        seed,
        mintX,
        mintY,
        lpAmount: new BN(userLpAmount),
        minX: new BN(userLpAmount),
        minY: new BN(userLpAmount),
      });

      const userLpAfter = await getAccount(connection, userLp);
      const lpMint = await getMint(connection, derivePoolPdas(seed).mintLp);
      expect(Number(userLpAfter.amount)).to.equal(0);
      expect(Number(lpMint.supply)).to.equal(MINIMUM_LIQUIDITY);
    });
  });

  describe("phase5: swap", () => {
    const FEE_BPS = 30;
    const FEE_DENOM = 10_000n;

    function expectedAmountOut(
      amountIn: bigint,
      reserveIn: bigint,
      reserveOut: bigint,
      feeBps: number
    ): bigint {
      const feeMultiplier = FEE_DENOM - BigInt(feeBps);
      const amountInWithFee = amountIn * feeMultiplier;
      const numerator = amountInWithFee * reserveOut;
      const denominator = reserveIn * FEE_DENOM + amountInWithFee;
      return numerator / denominator;
    }

    async function swapTokens(args: {
      seed: BN;
      mintX: PublicKey;
      mintY: PublicKey;
      isX: boolean;
      amountIn: BN;
      minOut: BN;
    }) {
      const { config } = derivePoolPdas(args.seed);
      const vaultX = getAssociatedTokenAddressSync(
        args.mintX,
        config,
        true,
        TOKEN_PROGRAM_ID
      );
      const vaultY = getAssociatedTokenAddressSync(
        args.mintY,
        config,
        true,
        TOKEN_PROGRAM_ID
      );
      const userX = getAssociatedTokenAddressSync(
        args.mintX,
        wallet.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );
      const userY = getAssociatedTokenAddressSync(
        args.mintY,
        wallet.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );

      const tx = await program.methods
        .swap(args.isX, args.amountIn, args.minOut)
        .accountsPartial({
          user: wallet.publicKey,
          mintX: args.mintX,
          mintY: args.mintY,
          config,
          vaultX,
          vaultY,
          userX,
          userY,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      return { tx, config, vaultX, vaultY, userX, userY };
    }

    async function setupSwapPool(seed: BN) {
      const [mintX, mintY] = await createPairMints();
      await initializePool({
        seed,
        fee: FEE_BPS,
        authority: null,
        mintX,
        mintY,
      });
      // Fund LP + extra for swapping
      await fundUserAtas(mintX, mintY, 5_000_000n, 5_000_000n);
      await depositLiquidity({
        seed,
        mintX,
        mintY,
        amountX: new BN(1_000_000),
        amountY: new BN(1_000_000),
        minLp: new BN(1_000_000 - MINIMUM_LIQUIDITY),
      });
      return { mintX, mintY };
    }

    it("swaps X→Y with fee and preserves k invariant", async () => {
      const seed = new BN(300);
      const { mintX, mintY } = await setupSwapPool(seed);
      const { config } = derivePoolPdas(seed);
      const vaultX = getAssociatedTokenAddressSync(mintX, config, true, TOKEN_PROGRAM_ID);
      const vaultY = getAssociatedTokenAddressSync(mintY, config, true, TOKEN_PROGRAM_ID);
      const userY = getAssociatedTokenAddressSync(
        mintY,
        wallet.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );

      const amountIn = 10_000n;
      const vaultXBefore = await getAccount(connection, vaultX);
      const vaultYBefore = await getAccount(connection, vaultY);
      const userYBefore = await getAccount(connection, userY);

      const expectedOut = expectedAmountOut(
        amountIn,
        vaultXBefore.amount,
        vaultYBefore.amount,
        FEE_BPS
      );

      await swapTokens({
        seed,
        mintX,
        mintY,
        isX: true,
        amountIn: new BN(amountIn.toString()),
        minOut: new BN(expectedOut.toString()),
      });

      const vaultXAfter = await getAccount(connection, vaultX);
      const vaultYAfter = await getAccount(connection, vaultY);
      const userYAfter = await getAccount(connection, userY);

      expect(vaultXAfter.amount).to.equal(vaultXBefore.amount + amountIn);
      expect(vaultYAfter.amount).to.equal(vaultYBefore.amount - expectedOut);
      expect(userYAfter.amount - userYBefore.amount).to.equal(expectedOut);

      const kBefore = vaultXBefore.amount * vaultYBefore.amount;
      const kAfter = vaultXAfter.amount * vaultYAfter.amount;
      expect(kAfter >= kBefore).to.equal(true);
    });

    it("swaps Y→X", async () => {
      const seed = new BN(301);
      const { mintX, mintY } = await setupSwapPool(seed);
      const { config } = derivePoolPdas(seed);
      const vaultX = getAssociatedTokenAddressSync(mintX, config, true, TOKEN_PROGRAM_ID);
      const vaultY = getAssociatedTokenAddressSync(mintY, config, true, TOKEN_PROGRAM_ID);
      const userX = getAssociatedTokenAddressSync(
        mintX,
        wallet.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );

      const amountIn = 5_000n;
      const vaultXBefore = await getAccount(connection, vaultX);
      const vaultYBefore = await getAccount(connection, vaultY);
      const userXBefore = await getAccount(connection, userX);
      const expectedOut = expectedAmountOut(
        amountIn,
        vaultYBefore.amount,
        vaultXBefore.amount,
        FEE_BPS
      );

      await swapTokens({
        seed,
        mintX,
        mintY,
        isX: false,
        amountIn: new BN(amountIn.toString()),
        minOut: new BN(expectedOut.toString()),
      });

      const userXAfter = await getAccount(connection, userX);
      expect(userXAfter.amount - userXBefore.amount).to.equal(expectedOut);
    });

    it("rejects slippage when min_amount_out is too high", async () => {
      const seed = new BN(302);
      const { mintX, mintY } = await setupSwapPool(seed);

      try {
        await swapTokens({
          seed,
          mintX,
          mintY,
          isX: true,
          amountIn: new BN(10_000),
          minOut: new BN(1_000_000),
        });
        expect.fail("expected SlippageExceeded");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/SlippageExceeded|6003|0x1773/i);
      }
    });

    it("rejects zero amount_in", async () => {
      const seed = new BN(303);
      const { mintX, mintY } = await setupSwapPool(seed);

      try {
        await swapTokens({
          seed,
          mintX,
          mintY,
          isX: true,
          amountIn: new BN(0),
          minOut: new BN(0),
        });
        expect.fail("expected InvalidAmount");
      } catch (err: unknown) {
        expect(errorBlob(err)).to.match(/InvalidAmount|6008|0x1778/i);
      }
    });

    it("fee reduces output versus zero-fee pool", async () => {
      // Pool A: fee 30, Pool B: fee 0 — same reserves/input → A out < B out
      const seedFee = new BN(304);
      const seedFree = new BN(305);

      const [mintX1, mintY1] = await createPairMints();
      await initializePool({
        seed: seedFee,
        fee: 30,
        authority: null,
        mintX: mintX1,
        mintY: mintY1,
      });
      await fundUserAtas(mintX1, mintY1, 2_000_000n, 2_000_000n);
      await depositLiquidity({
        seed: seedFee,
        mintX: mintX1,
        mintY: mintY1,
        amountX: new BN(1_000_000),
        amountY: new BN(1_000_000),
        minLp: new BN(1_000_000 - MINIMUM_LIQUIDITY),
      });

      const [mintX2, mintY2] = await createPairMints();
      await initializePool({
        seed: seedFree,
        fee: 0,
        authority: null,
        mintX: mintX2,
        mintY: mintY2,
      });
      await fundUserAtas(mintX2, mintY2, 2_000_000n, 2_000_000n);
      await depositLiquidity({
        seed: seedFree,
        mintX: mintX2,
        mintY: mintY2,
        amountX: new BN(1_000_000),
        amountY: new BN(1_000_000),
        minLp: new BN(1_000_000 - MINIMUM_LIQUIDITY),
      });

      const amountIn = 10_000n;
      const outFee = expectedAmountOut(amountIn, 1_000_000n, 1_000_000n, 30);
      const outFree = expectedAmountOut(amountIn, 1_000_000n, 1_000_000n, 0);
      expect(outFee < outFree).to.equal(true);

      const userY1 = getAssociatedTokenAddressSync(
        mintY1,
        wallet.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );
      const userY2 = getAssociatedTokenAddressSync(
        mintY2,
        wallet.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );
      const y1Before = await getAccount(connection, userY1);
      const y2Before = await getAccount(connection, userY2);

      await swapTokens({
        seed: seedFee,
        mintX: mintX1,
        mintY: mintY1,
        isX: true,
        amountIn: new BN(amountIn.toString()),
        minOut: new BN(outFee.toString()),
      });
      await swapTokens({
        seed: seedFree,
        mintX: mintX2,
        mintY: mintY2,
        isX: true,
        amountIn: new BN(amountIn.toString()),
        minOut: new BN(outFree.toString()),
      });

      const y1After = await getAccount(connection, userY1);
      const y2After = await getAccount(connection, userY2);
      expect(y1After.amount - y1Before.amount).to.equal(outFee);
      expect(y2After.amount - y2Before.amount).to.equal(outFree);
    });
  });
});
