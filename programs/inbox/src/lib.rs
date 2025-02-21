use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    secp256k1_recover::Secp256k1Pubkey,
    program_pack::Pack,
};
use borsh::{BorshSerialize, BorshDeserialize};

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

// `Sealed` trait'i boş bir trait'tir ve `Pack` trait'inin bir gereksinimidir.
// Bu trait, tipin durumunu değiştirmeyi güvenli bir şekilde kontrol etmek için kullanılır.
impl solana_program::program_pack::Sealed for InboxAccount {}

impl Pack for InboxAccount {
    const LEN: usize = {
        // Sabit boyutlu alanların toplam boyutu:
        // - records: Vector olduğu için, eleman sayısını tutan u32 (4 byte)
        // - index: u32 (4 byte)
        let constant_size = 4 + 4;

        // Her bir FileTxRecord için:
        // - sender_ec_pubkey: [u8; 33] (33 byte)
        // - encrypted_link: Vec<u8> için eleman sayısını tutan u32 (4 byte) + maksimum 1024 byte veri
        // - ephemeral_pubkey: [u8; 33] (33 byte)
        let record_size = 33 + 4 + 1024 + 33;

        // Maksimum 10 kayıt için toplam boyut:
        constant_size + (record_size * 10)
    };


    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut data = Vec::with_capacity(Self::LEN);
        // `BorshSerialize` trait'ini kullanarak serileştirme
        self.serialize(&mut data).expect("Failed to serialize InboxAccount");

        // Hedef dilime kopyala
        dst.copy_from_slice(&data);
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        // `BorshDeserialize` trait'ini kullanarak deserializasyon
        Self::try_from_slice(src).map_err(|_| ProgramError::InvalidAccountData)
    }
    
    fn get_packed_len() -> usize {
        Self::LEN
    }
    
    fn unpack(input: &[u8]) -> Result<Self, ProgramError>
    where
        Self: solana_program::program_pack::IsInitialized,
    {
        let value = Self::unpack_unchecked(input)?;
        if value.is_initialized() {
            Ok(value)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    }
    
    fn unpack_unchecked(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Self::unpack_from_slice(input)
    }
    
    fn pack(src: Self, dst: &mut [u8]) -> Result<(), ProgramError> {
        if dst.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        src.pack_into_slice(dst);
        Ok(())
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
        msg!("Geçersiz PDA adresi");
        return Err(ProgramError::InvalidAccountData);
    }

    // Kayıt deserialization
    let record = FileTxRecord::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Boyut kontrolü
    if record.encrypted_link.len() > 1024 {
        msg!("Şifreli link boyutu aşıldı");
        return Err(ProgramError::InvalidInstructionData);
    }

    // EC pubkey doğrulama (Güncel Solana SDK yapısına göre)
    let _ec_pubkey = Secp256k1Pubkey::new(&record.ephemeral_pubkey)
        .map_err(|_| {
            msg!("Geçersiz EC public key");
            ProgramError::InvalidArgument
        })?;

    // Hesap verisini güncelle
    let mut account_data = InboxAccount::unpack_from_slice(&inbox_account.data.borrow())?;
    account_data.records.push(record);
    account_data.index = account_data.index.checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    
    InboxAccount::pack_into_slice(&account_data, &mut inbox_account.data.borrow_mut());

    Ok(())
} 