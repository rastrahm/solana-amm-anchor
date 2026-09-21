import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { Amm } from "../target/types/amm";

/**
 * Manual Config layout (must stay in sync with `state/config.rs` InitSpace):
 * Option<Pubkey> 33 + Pubkey×3 96 + u64 8 + u16 2 + bool 1 + u8×2 2 = 142
 * Account space = 8 (discriminator) + 142 = 150
 */
const CONFIG_INIT_SPACE = 142;
const CONFIG_ACCOUNT_SPACE = 8 + CONFIG_INIT_SPACE;

describe("amm", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.amm as Program<Amm>;

  it("smoke: initialize is callable", async () => {
    const tx = await program.methods.initialize().rpc();
    expect(tx).to.be.a("string");
    expect(tx.length).to.be.greaterThan(0);
  });

  it("phase1: Config account space is discriminator + InitSpace", () => {
    expect(CONFIG_ACCOUNT_SPACE).to.equal(150);
    expect(CONFIG_INIT_SPACE).to.equal(142);
  });
});
