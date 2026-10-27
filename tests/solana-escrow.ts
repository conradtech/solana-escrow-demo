import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaEscrow } from "../target/types/solana_escrow";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  getAccount,
} from "@solana/spl-token";
import { Keypair, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("solana-escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaEscrow as Program<SolanaEscrow>;
  const connection = provider.connection;
  const maker = (provider.wallet as anchor.Wallet).payer;

  async function setupEscrow() {
    const mintA = await createMint(connection, maker, maker.publicKey, null, 0);
    const mintB = await createMint(connection, maker, maker.publicKey, null, 0);

    const makerAtaA = (await getOrCreateAssociatedTokenAccount(
      connection, maker, mintA, maker.publicKey
    )).address;

    const makerAtaB = (await getOrCreateAssociatedTokenAccount(
      connection, maker, mintB, maker.publicKey
    )).address;

    await mintTo(connection, maker, mintA, makerAtaA, maker, 1000);

    const seed = new anchor.BN(Date.now() + Math.floor(Math.random() * 100000));
    const amountA = new anchor.BN(100);
    const amountB = new anchor.BN(200);

    const [escrow] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        maker.publicKey.toBuffer(),
        seed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const vault = await anchor.utils.token.associatedAddress({
      mint: mintA,
      owner: escrow,
    });

    await program.methods
      .make(seed, amountA, amountB)
      .accounts({
        maker: maker.publicKey,
        mintA,
        mintB,
        makerAtaA,
        escrow,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    return {
      mintA, mintB, makerAtaA, makerAtaB, seed, escrow, vault,
      amountA, amountB,
    };
  }

  it("creates an escrow and deposits token A", async () => {
    const { vault } = await setupEscrow();
    const vaultAccount = await getAccount(connection, vault);
    assert.equal(vaultAccount.amount, BigInt(100));
  });

  it("lets a taker pay token B and receive token A", async () => {
    const state = await setupEscrow();
    const taker = Keypair.generate();

    const signature = await connection.requestAirdrop(
      taker.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(signature, "confirmed");

    const takerAtaA = (await getOrCreateAssociatedTokenAccount(
      connection, maker, state.mintA, taker.publicKey
    )).address;

    const takerAtaB = (await getOrCreateAssociatedTokenAccount(
      connection, maker, state.mintB, taker.publicKey
    )).address;

    await mintTo(connection, maker, state.mintB, takerAtaB, maker, 1000);

    await program.methods
      .take()
      .accounts({
        taker: taker.publicKey,
        maker: maker.publicKey,
        escrow: state.escrow,
        mintA: state.mintA,
        mintB: state.mintB,
        vault: state.vault,
        takerAtaA,
        takerAtaB,
        makerAtaB: state.makerAtaB,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([taker])
      .rpc();

    const takerA = await getAccount(connection, takerAtaA);
    const takerB = await getAccount(connection, takerAtaB);
    const makerB = await getAccount(connection, state.makerAtaB);

    assert.equal(takerA.amount, BigInt(100));
    assert.equal(takerB.amount, BigInt(800));
    assert.equal(makerB.amount, BigInt(200));
  });
});
