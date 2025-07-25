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

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
