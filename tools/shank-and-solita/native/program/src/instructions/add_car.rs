use {
    borsh::{
        BorshDeserialize,
        BorshSerialize,
    },
    shank::ShankAccount,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
        sysvar::Sysvar,
    },
};
use crate::state::Car;

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct AddCarArgs {
    pub year: u16,
    pub make: String,
    pub model: String,
}

pub fn add_car(program_id: &Pubkey, accounts: &[AccountInfo], args: AddCarArgs) -> ProgramResult {
    let AddCarArgs { year, make, model } = args;

    let accounts_iter = &mut accounts.iter();
    let car_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let (car_account_pda, car_account_bump) = Car::shank_pda(program_id, make.clone(), model.clone());
    if &car_account_pda != car_account.key {
        // Avoid panics in on-chain programs; return a deterministic error instead.
        return Err(ProgramError::InvalidSeeds);
    }

    let car_data = Car {
        year,
        make: make.clone(),
        model: model.clone(),
    };

    let account_span = (car_data.try_to_vec()?).len();
    let lamports_required = (Rent::get()?).minimum_balance(account_span);

    invoke_signed(
        &system_instruction::create_account(
            &payer.key,
            &car_account.key,
            lamports_required,
            account_span as u64,
            program_id,
        ),
        &[payer.clone(), car_account.clone(), system_program.clone()],
        Car::shank_seeds_with_bump(make, model, &[car_account_bump]),
    )?;

    car_data.serialize(&mut &mut car_account.data.borrow_mut()[..])?;

    Ok(())
}
