import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import {
  createMint,
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

describe("amm", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.amm as Program<Amm>;
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  it("phase1: Config account space is discriminator + InitSpace", () => {
    expect(CONFIG_ACCOUNT_SPACE).to.equal(150);
    expect(CONFIG_INIT_SPACE).to.equal(142);
  });

  describe("phase2: initialize", () => {
    let mintX: PublicKey;
    let mintY: PublicKey;

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

    function derivePoolPdas(seed: BN): {
      config: PublicKey;
      configBump: number;
      mintLp: PublicKey;
      lpBump: number;
    } {
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
});
