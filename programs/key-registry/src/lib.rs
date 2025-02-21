use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    secp256k1_recover::Secp256k1Pubkey,
};
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ZklAccount {
    pub is_initialized: bool,
    #[borsh(serialize_with = "serialize_pubkey")]
    #[borsh(deserialize_with = "deserialize_pubkey")]
    pub ec_pubkey: [u8; 33],
    pub zkl_sig: [u8; 64],   // Signature with EC private key
    pub solana_sig: [u8; 64], // Signature with Solana private key
    pub index: u32,
}

fn serialize_pubkey<W: std::io::Write>(
    pubkey: &[u8; 33],
    writer: &mut W,
) -> std::io::Result<()> {
    borsh::BorshSerialize::serialize(pubkey, writer)
}

fn deserialize_pubkey<R: std::io::Read>(
    reader: &mut R,
) -> std::io::Result<[u8; 33]> {
    borsh::BorshDeserialize::deserialize(reader)
}

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let zkl_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // PDA validation
    let (pda, bump_seed) = Pubkey::find_program_address(
        &[b"zkl_account", payer.key.as_ref()],
        program_id,
    );
    if pda != *zkl_account.key {
        msg!("Invalid PDA for zkl account");
        return Err(ProgramError::InvalidAccountData);
    }

    // Deserialize input data
    let input_data = ZklAccount::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Verify EC signature
    let ec_pubkey = Secp256k1Pubkey::new_from_compressed(input_data.ec_pubkey)
        .map_err(|_| ProgramError::InvalidArgument)?;
    let message = format!("ZKLAccount:{}", pda).into_bytes();
    ec_pubkey.verify(&message, &input_data.zkl_sig)
        .map_err(|_| ProgramError::InvalidArgument)?;

    // Verify Solana signature
    let solana_sig_valid = payer.verify_signature(
        &message,
        &input_data.solana_sig,
    );
    if !solana_sig_valid {
        return Err(ProgramError::InvalidArgument);
    }

    // Initialize account
    let mut account_data = ZklAccount::try_from_slice(&zkl_account.data.borrow())?;
    if account_data.is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.is_initialized = true;
    account_data.ec_pubkey = input_data.ec_pubkey;
    account_data.zkl_sig = input_data.zkl_sig;
    account_data.solana_sig = input_data.solana_sig;
    account_data.index = input_data.index;

    ZklAccount::pack(account_data, &mut zkl_account.data.borrow_mut())?;

    Ok(())
} 