#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5ECuja28Pr3QuBjmwke2NVcJPrbs47AXrFAUbE4NsP2n");

#[program]
pub mod dice_game {
    use super::*;

    //initializing the house and transfering the some sol to house's vault
    pub fn initialize(ctx: Context<Initialize> , amount:u64) -> Result<()> {
        ctx.accounts.initialize_house(amount)?;
        Ok(())
    }

    pub fn place_bet(ctx: Context<PlaceBet> , seed: u128, roll: u8, amount: u64,) -> Result<()>{
        ctx.accounts.create_bet(seed, roll, amount, &ctx.bumps)?;
        Ok(())
    }

    pub fn refund(ctx: Context<RefundBet>) -> Result<()>{
        ctx.accounts.refund_bet(&ctx.bumps)?;
        Ok(())
    }

    pub fn resolve_bet(ctx: Context<ResolveBet> , sig:Vec<u8>) -> Result<()>{
        ctx.accounts.verify_ed25519_signature(&sig)?;
        ctx.accounts.resolve_bet(&sig)?;
        Ok(())
    } 
}
