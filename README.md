# Solana SPL Token Escrow (Rust + Anchor)

An educational Anchor program demonstrating a two-token escrow flow on Solana. The maker deposits token A into a PDA-controlled associated token account; a taker pays token B to the maker and receives token A. If the trade has not been taken, the maker can refund token A.

> **Status: starter implementation, not audited.** Do not deploy with real funds or present this as production-ready. Run the build and tests locally, fix any toolchain issues, and add full integration/security tests before sharing it as verified work.

## What it demonstrates

- Rust program instructions using Anchor.
- PDA-derived escrow state and vault authority.
- SPL Token `transfer_checked` CPIs.
- Signer, mint, owner, and associated-token-account constraints.
- Maker refund path and account closure.
- A starter TypeScript test harness.

## Requirements

Install compatible versions of:

- Rust and Cargo
- Solana CLI
- Anchor CLI 0.30.1
- Node.js and Yarn or npm

Version details change; use the official Solana and Anchor installation guides to install a compatible toolchain.

## Build and test

1. Replace the placeholder program ID in both `Anchor.toml` and `programs/solana_escrow/src/lib.rs` with the ID generated for your program.
2. Install JavaScript dependencies: `npm install`.
3. Generate the program keypair / IDL as required by your Anchor version: `anchor keys sync`.
4. Build: `anchor build`.
5. Run local validator tests: `anchor test`.

The included test is a smoke-test starter, not a complete end-to-end escrow test. Before claiming the project is fully tested, add fixtures that create two mints and associated token accounts, mint test tokens to maker/taker, then test successful take, maker refund, wrong signer, wrong mint, insufficient balance, and duplicate/closed escrow behavior.

## Important design notes / limitations

- This is a teaching sample, not an audited protocol.
- It assumes the classic SPL Token program, not Token-2022 extensions.
- There is no timeout or cancellation delay; maker can refund while the escrow exists.
- Production work should add robust integration tests, review account constraints and CPI behavior, define supported token semantics, and obtain an independent security review.
- Confirm the program ID and generated TypeScript type names after `anchor build`; they depend on your local Anchor version and IDL generation.

## How to describe it honestly

Once you have personally built, tested, and can explain it, you can describe it as an independent educational project. Do not claim it is deployed, audited, or production-ready unless those statements are true.
