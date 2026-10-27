import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaEscrow } from "../target/types/solana_escrow";

describe("solana-escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .SolanaEscrow as Program<SolanaEscrow>;

  it("loads the escrow program", async () => {
    console.log("Escrow program ID:", program.programId.toBase58());
  });
});
