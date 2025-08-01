use anchor_instruction_sysvar::Ed25519InstructionSignatures;
use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use solana_program::{
    blake3::hash, ed25519_program, sysvar::instructions::load_instruction_at_checked,
};

use crate::{error::DiceError, Bet};

pub const HOUSE_EDGE: u16 = 150;

#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub house: Signer<'info>,
    #[account(mut)]
    /// CHECK: This is safe
    pub player: UncheckedAccount<'info>,
    /// CHECK: House is checked Manually
    #[account(
        mut,
        seeds = [b"vault" , house.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        mut,
        close = player,
        seeds = [b"bet" , vault.key().as_ref() , bet.seed.to_le_bytes().as_ref()],
        bump=bet.bump,
    )]
    pub bet: Account<'info, Bet>,
    #[account(
        address = solana_program::sysvar::instructions::ID
    )]
    /// CHECK: This is safe
    pub instruction_sysvar: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn verify_ed25519_signature(&mut self, sig: &[u8]) -> Result<()> {
        //get the ed25519 signature
        let ix = load_instruction_at_checked(0, &self.instruction_sysvar.to_account_info())?;

        //program id of our ix should be match with ed25519d key
        require_keys_eq!(
            ix.program_id,
            ed25519_program::ID,
            DiceError::Ed25519Program
        );

        //in signature there should be no account present
        require_eq!(ix.accounts.len(), 0, DiceError::Ed25519Accounts);

        //structured
        let signatures = Ed25519InstructionSignatures::unpack(&ix.data)?.0;

        //there should be only one signature
        require_eq!(signatures.len(), 1, DiceError::Ed25519DataLength);
        let signature = &signatures[0];

        // data should be verifiabel which is present in signature
        require!(signature.is_verifiable, DiceError::Ed25519Header);

        //public should be matches keys match
        require_keys_eq!(
            signature.public_key.ok_or(DiceError::Ed25519Pubkey)?,
            self.house.key(),
            DiceError::Ed25519Pubkey
        );

        // Ensure signatures match
        require!(
            &signature
                .signature
                .ok_or(DiceError::Ed25519Signature)?
                .eq(sig),
            DiceError::Ed25519Signature
        );

        // Ensure messages match
        require!(
            &signature
                .message
                .as_ref()
                .ok_or(DiceError::Ed25519Signature)?
                .eq(&self.bet.to_slice()),
            DiceError::Ed25519Signature
        );

        Ok(())
    }

    pub fn resolve_bet(&mut self,sig: &[u8]) -> Result<()> {
        let hash = hash(sig).to_bytes();
        let mut hash_16: [u8; 16] = [0; 16];
        hash_16.copy_from_slice(&hash[0..16]);
        let lower = u128::from_le_bytes(hash_16);
        hash_16.clone_from_slice(&hash[16..32]);
        let upper = u128::from_le_bytes(hash_16);

        let roll = lower.wrapping_add(upper).wrapping_rem(100) as u8 + 1;

        if self.bet.roll > roll {
            msg!("player won");
            let payout = (self.bet.amount as u128)
                .checked_mul(10000 - HOUSE_EDGE as u128).ok_or(DiceError::Overflow)?
                .checked_div(self.bet.roll as u128 - 1).ok_or(DiceError::Overflow)?
                .checked_div(100).ok_or(DiceError::Overflow)? as u64;

            let seed = &[
                &b"vault"[..],
                &self.house.key().to_bytes(),
                &[self.bet.bump]
            ];
            let signer_seeds = &[&seed[..]];
            let cpi_context = CpiContext::new_with_signer(self.system_program.to_account_info(), Transfer{
                from:self.vault.to_account_info(),
                to:self.player.to_account_info()
            } , signer_seeds);
            transfer(cpi_context, payout)?;
        }

        Ok(())
    }
}
