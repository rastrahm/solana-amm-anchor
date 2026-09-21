import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { Amm } from "../target/types/amm";

describe("amm", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.amm as Program<Amm>;

  it("smoke: initialize is callable", async () => {
    const tx = await program.methods.initialize().rpc();
    expect(tx).to.be.a("string");
    expect(tx.length).to.be.greaterThan(0);
  });
});
