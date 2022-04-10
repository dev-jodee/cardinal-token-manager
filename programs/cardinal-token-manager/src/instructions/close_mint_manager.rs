use {
    crate::{errors::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::{self, Mint, SetAuthority, Token},
    spl_token::instruction::AuthorityType,
};

#[derive(Accounts)]
pub struct CloseMintManagerCtx<'info> {
    #[account(constraint = mint_manager.token_managers == 0 @ ErrorCode::OutstandingTokens)]
    pub mint_manager: Account<'info, MintManager>,
    #[account(mut, close = freeze_authority)]
    pub mint: Account<'info, Mint>,
    #[account(constraint = mint_manager.initializer == freeze_authority.key() @ ErrorCode::InvalidInitializer)]
    pub freeze_authority: Signer<'info>,
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CloseMintManagerCtx>) -> Result<()> {
    if !ctx.accounts.mint.freeze_authority.is_some() || ctx.accounts.mint.freeze_authority.unwrap() != ctx.accounts.mint_manager.key() {
        return Err(error!(ErrorCode::InvalidFreezeAuthority));
    }

    // get PDA seeds to sign with
    let mint = ctx.accounts.mint.key();
    let mint_manager_seeds = &[MINT_MANAGER_SEED.as_bytes(), mint.as_ref(), &[ctx.accounts.mint_manager.bump]];
    let mint_manager_signer = &[&mint_manager_seeds[..]];

    // set freeze authority of mint back to original
    let cpi_accounts = SetAuthority {
        account_or_mint: ctx.accounts.freeze_authority.to_account_info(),
        current_authority: ctx.accounts.mint_manager.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(mint_manager_signer);
    token::set_authority(cpi_context, AuthorityType::FreezeAccount, Some(ctx.accounts.freeze_authority.key()))?;
    Ok(())
}
