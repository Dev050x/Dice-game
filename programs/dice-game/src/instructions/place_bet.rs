use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

use crate::Bet;

#[derive(Accounts)]
#[instruction(seed:u128)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    ///Check: This is safe
    pub house: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"vault" , house.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        init,
        payer = player,
        space= Bet::INIT_SPACE + 8,
        seeds = [b"bet" , vault.key().as_ref() , seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>,
}

impl<'info> PlaceBet<'info> {
    pub fn create_bet(&mut self, seed:u128 , roll: u8 , amount:u64 , bumps:&PlaceBetBumps) -> Result<()> {
        self.bet.set_inner(Bet {
            player: self.player.key(),
            seed,
            amount,
            slot: Clock::get()?.slot,
            roll,
            bump: bumps.bet,
        });

        let cpiContext = CpiContext::new(self.system_program.to_account_info(), Transfer{
            from:self.player.to_account_info(),
            to:self.vault.to_account_info()
        });
        transfer(cpiContext, amount)?;

        Ok(())
    }
}
