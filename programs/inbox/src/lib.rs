use solana_program::{
    borsh::try_from_slice_unchecked,
    program_error::ProgramError,
    secp256k1_recover::Secp256k1Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct FileTxRecord {
    pub sender_ec_pubkey: [u8; 33],
    pub encrypted_link: Vec<u8>,
    pub ephemeral_pubkey: [u8; 33],
    pub timestamp: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InboxAccount {
    pub records: Vec<FileTxRecord>,
    pub index: u32,
}

pub fn add_file_record(
    inbox_account: &mut InboxAccount,
    record: FileTxRecord,
) -> ProgramResult {
    // Validate record size
    if record.encrypted_link.len() > 1024 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Verify ephemeral public key
    let _ = Secp256k1Pubkey::new_from_compressed(record.ephemeral_pubkey)
        .map_err(|_| ProgramError::InvalidArgument)?;

    inbox_account.records.push(record);
    Ok(())
} 