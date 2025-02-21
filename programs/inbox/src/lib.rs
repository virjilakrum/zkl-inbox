use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    secp256k1_recover::Secp256k1Pubkey,
    program_pack::{Pack, Sealed},
};
use borsh::{BorshSerialize, BorshDeserialize};
use libsecp256k1;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct FileTxRecord {
    pub sender_ec_pubkey: [u8; 33],
    pub encrypted_link: Vec<u8>,
    pub ephemeral_pubkey: [u8; 33],
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InboxAccount {
    pub records: Vec<FileTxRecord>,
    pub index: u32,
}

impl Sealed for InboxAccount {}

impl Pack for InboxAccount {
    const LEN: usize = 1098;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        borsh::to_writer(&mut dst[..], self).unwrap();
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        borsh::from_slice(src).map_err(|_| ProgramError::InvalidAccountData)
    }
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let inbox_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;

    // PDA doğrulama
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"inbox", payer.key.as_ref()],
        program_id,
    );
    
    if pda != *inbox_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    // Kayıt deserialization
    let record: FileTxRecord = borsh::from_slice(instruction_data)?;

    // Boyut kontrolü
    if record.encrypted_link.len() > 1024 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // EC pubkey doğrulama
    let _ = libsecp256k1::PublicKey::parse_compressed(&record.ephemeral_pubkey)
        .map_err(|_| ProgramError::InvalidArgument)?;

    // Hesap verisini güncelle
    let mut account_data = InboxAccount::unpack_from_slice(&inbox_account.data.borrow())?;
    account_data.records.push(record);
    account_data.index = account_data.index.saturating_add(1);
    
    InboxAccount::pack_into_slice(&account_data, &mut inbox_account.data.borrow_mut());

    Ok(())
} 