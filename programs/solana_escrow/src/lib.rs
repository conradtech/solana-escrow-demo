use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, TransferChecked};

// Replace this placeholder with the program ID generated for your own deployment.
declare_id!("EXN6vnn9RhSj9jqMLMX7U3o2rnUbwm4L8dXLFmyVgkBa");

#[program]
pub mod solana_escrow {
    use super::*;

    /// Maker deposits `amount_a` of mint A into a vault controlled by this escrow PDA.
    pub fn make(ctx: Context<Make>, seed: u64, amount_a: u64, amount_b: u64) -> Result<()> {
        require!(amount_a > 0 && amount_b > 0, EscrowError::ZeroAmount);
        require!(ctx.accounts.mint_a.key() != ctx.accounts.mint_b.key(), EscrowError::SameMint);

        let escrow = &mut ctx.accounts.escrow;
        escrow.maker = ctx.accounts.maker.key();
        escrow.mint_a = ctx.accounts.mint_a.key();
        escrow.mint_b = ctx.accounts.mint_b.key();
        escrow.amount_a = amount_a;
        escrow.amount_b = amount_b;
        escrow.seed = seed;
        escrow.bump = ctx.bumps.escrow;

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.maker_ata_a.to_account_info(),
            mint: ctx.accounts.mint_a.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.maker.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer_checked(cpi_ctx, amount_a, ctx.accounts.mint_a.decimals)?;
        Ok(())
    }

    /// Taker pays mint B to the maker and receives the escrowed mint A.
    pub fn take(ctx: Context<Take>) -> Result<()> {
        let amount_a = ctx.accounts.escrow.amount_a;
        let amount_b = ctx.accounts.escrow.amount_b;
        let decimals_b = ctx.accounts.mint_b.decimals;
        let decimals_a = ctx.accounts.mint_a.decimals;
        let seed = ctx.accounts.escrow.seed.to_le_bytes();
        let bump = [ctx.accounts.escrow.bump];
        let signer_seeds: &[&[u8]] = &[b"escrow", ctx.accounts.escrow.maker.as_ref(), seed.as_ref(), bump.as_ref()];
        let signer = &[signer_seeds];

        token::transfer_checked(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), TransferChecked {
                from: ctx.accounts.taker_ata_b.to_account_info(),
                mint: ctx.accounts.mint_b.to_account_info(),
                to: ctx.accounts.maker_ata_b.to_account_info(),
                authority: ctx.accounts.taker.to_account_info(),
            }), amount_b, decimals_b,
        )?;

        token::transfer_checked(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
                to: ctx.accounts.taker_ata_a.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            }, signer), amount_a, decimals_a,
        )?;

        token::close_account(CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.maker.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        }, signer))?;
        // `escrow` is closed to maker by the account constraint after this instruction.
        Ok(())
    }

    /// Maker reclaims the deposited tokens if the trade has not been taken.
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        let amount_a = ctx.accounts.escrow.amount_a;
        let decimals_a = ctx.accounts.mint_a.decimals;
        let seed = ctx.accounts.escrow.seed.to_le_bytes();
        let bump = [ctx.accounts.escrow.bump];
        let signer_seeds: &[&[u8]] = &[b"escrow", ctx.accounts.escrow.maker.as_ref(), seed.as_ref(), bump.as_ref()];
        let signer = &[signer_seeds];

        token::transfer_checked(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
                to: ctx.accounts.maker_ata_a.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            }, signer), amount_a, decimals_a,
        )?;
        token::close_account(CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.maker.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        }, signer))?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    #[account(mut, constraint = maker_ata_a.mint == mint_a.key(), constraint = maker_ata_a.owner == maker.key())]
    pub maker_ata_a: Account<'info, TokenAccount>,
    #[account(init, payer = maker, space = 8 + Escrow::INIT_SPACE, seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()], bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(init, payer = maker, associated_token::mint = mint_a, associated_token::authority = escrow)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    /// CHECK: The maker address is stored in the program-owned escrow account and constrained below.
    #[account(mut, address = escrow.maker)]
    pub maker: UncheckedAccount<'info>,
    #[account(mut, close = maker, seeds = [b"escrow", escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()], bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(address = escrow.mint_a)]
    pub mint_a: Account<'info, Mint>,
    #[account(address = escrow.mint_b)]
    pub mint_b: Account<'info, Mint>,
    #[account(mut, associated_token::mint = mint_a, associated_token::authority = escrow)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = taker_ata_a.mint == mint_a.key(), constraint = taker_ata_a.owner == taker.key())]
    pub taker_ata_a: Account<'info, TokenAccount>,
    #[account(mut, constraint = taker_ata_b.mint == mint_b.key(), constraint = taker_ata_b.owner == taker.key())]
    pub taker_ata_b: Account<'info, TokenAccount>,
    #[account(mut, constraint = maker_ata_b.mint == mint_b.key(), constraint = maker_ata_b.owner == maker.key())]
    pub maker_ata_b: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut, address = escrow.maker)]
    pub maker: Signer<'info>,
    #[account(mut, close = maker, seeds = [b"escrow", escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()], bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(address = escrow.mint_a)]
    pub mint_a: Account<'info, Mint>,
    #[account(mut, associated_token::mint = mint_a, associated_token::authority = escrow)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = maker_ata_a.mint == mint_a.key(), constraint = maker_ata_a.owner == maker.key())]
    pub maker_ata_a: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub seed: u64,
    pub bump: u8,
}

#[error_code]
pub enum EscrowError {
    #[msg("Amounts must be greater than zero.")]
    ZeroAmount,
    #[msg("Escrow must exchange two different mints.")]
    SameMint,
}
