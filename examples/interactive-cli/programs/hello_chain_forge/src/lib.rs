//! Hello Chain Forge - A simple example Solana program
//!
//! This program demonstrates basic Solana program structure:
//! - Processing instructions
//! - Logging messages
//! - Modifying account data (counter)
//!
//! Instructions:
//! - 0: Initialize counter to 0
//! - 1: Increment counter by 1
//! - 2: Log "Hello, Chain Forge!"
//! - 3: Read and log current counter value

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Declare the program entrypoint
entrypoint!(process_instruction);

/// Program entrypoint
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello Chain Forge program invoked");

    // Get instruction type from first byte
    let instruction = instruction_data.first().unwrap_or(&2);

    match instruction {
        // Initialize: Set counter to 0
        0 => {
            msg!("Instruction: Initialize");
            let accounts_iter = &mut accounts.iter();
            let counter_account = next_account_info(accounts_iter)?;

            // Verify the account is owned by this program
            if counter_account.owner != program_id {
                msg!("Counter account not owned by program");
                return Err(ProgramError::IncorrectProgramId);
            }

            // Initialize counter to 0
            let mut data = counter_account.try_borrow_mut_data()?;
            if data.len() >= 8 {
                data[..8].copy_from_slice(&0u64.to_le_bytes());
                msg!("Counter initialized to 0");
            }
        }

        // Increment: Add 1 to counter
        1 => {
            msg!("Instruction: Increment");
            let accounts_iter = &mut accounts.iter();
            let counter_account = next_account_info(accounts_iter)?;

            // Verify the account is owned by this program
            if counter_account.owner != program_id {
                msg!("Counter account not owned by program");
                return Err(ProgramError::IncorrectProgramId);
            }

            // Read current value
            let mut data = counter_account.try_borrow_mut_data()?;
            if data.len() >= 8 {
                let mut value = u64::from_le_bytes(data[..8].try_into().unwrap());
                value = value.saturating_add(1);
                data[..8].copy_from_slice(&value.to_le_bytes());
                msg!("Counter incremented to: {}", value);
            }
        }

        // Read: Log current counter value (read-only)
        3 => {
            msg!("Instruction: Read");
            let accounts_iter = &mut accounts.iter();
            let counter_account = next_account_info(accounts_iter)?;

            // Verify the account is owned by this program
            if counter_account.owner != program_id {
                msg!("Counter account not owned by program");
                return Err(ProgramError::IncorrectProgramId);
            }

            // Read and log current value
            let data = counter_account.try_borrow_data()?;
            if data.len() >= 8 {
                let value = u64::from_le_bytes(data[..8].try_into().unwrap());
                msg!("Current counter value: {}", value);
            } else {
                msg!("Account data too small to read counter");
            }
        }

        // Hello: Just log a message (no accounts needed)
        _ => {
            msg!("Hello, Chain Forge!");
            msg!("Program ID: {}", program_id);
            msg!("Number of accounts: {}", accounts.len());

            // Log each account's pubkey
            for (i, account) in accounts.iter().enumerate() {
                msg!("Account {}: {}", i, account.key);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_parsing() {
        // Test that instruction bytes are parsed correctly
        assert_eq!(*[0u8].first().unwrap(), 0);
        assert_eq!(*[1u8].first().unwrap(), 1);
        assert_eq!(*[2u8].first().unwrap(), 2);
    }
}
