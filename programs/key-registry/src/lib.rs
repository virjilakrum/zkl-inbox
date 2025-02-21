use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint,
    entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError, 
    program_pack::{Pack, Sealed}, 
    pubkey::Pubkey, 
    rent::Rent, 
    secp256k1_recover::secp256k1_recover
};

use borsh::{BorshSerialize, BorshDeserialize};
use sha2::{Sha256, Digest};
use bs58;
use bytemuck;
use solana_program::ed25519_program;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ZklAccount {
    pub is_initialized: bool,      // 1 byte
    pub ec_pubkey: [u8; 64],       // Secp256k1 compressed pubkey (33 bytes) + padding
    pub zkl_sig: [u8; 64],         // ECDSA signature
    pub solana_sig: [u8; 64],      // Ed25519 signature
    pub index: u32,                // 4 bytes
}

impl Sealed for ZklAccount {}

impl Pack for ZklAccount {
    const LEN: usize = 1 + 64 + 64 + 64 + 4; // 197 bytes

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
    
    // Account validation
    let zkl_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // PDA derivation
    let (pda, bump_seed) = Pubkey::find_program_address(
        &[b"zkl_account", payer.key.as_ref()],
        program_id,
    );
    
    if pda != *zkl_account.key {
        msg!("Invalid PDA address");
        return Err(ProgramError::InvalidAccountData);
    }

    // Deserialize input
    let input_data: ZklAccount = ZklAccount::unpack_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Message construction
    let message = format!("ZKLAccount:{}", pda);
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    let message_hash = hasher.finalize();

    // ECDSA verification
    let recovery_id = input_data.zkl_sig[64]; // Last byte as recovery ID
    let signature = &input_data.zkl_sig[..64];
    
    let recovered_pubkey = secp256k1_recover(
        &message_hash,
        recovery_id,
        signature
    ).map_err(|_| ProgramError::InvalidArgument)?;

    // Public key comparison
    if recovered_pubkey.to_bytes() != input_data.ec_pubkey[..33] {
        msg!("EC Public key mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    // Hesap oluşturma işlemi PDA için düzeltildi
    let instruction = solana_program::system_instruction::create_account_with_seed(
        payer.key,
        zkl_account.key,
        program_id,
        &bs58::encode(bump_seed.to_le_bytes()).into_string(),
        Rent::default().minimum_balance(ZklAccount::LEN),
        ZklAccount::LEN as u64,
        program_id,
    );

    solana_program::program::invoke(
        &instruction,
        &[payer.clone(), zkl_account.clone(), system_program.clone()],
    ).map_err(|_| ProgramError::InvalidArgument)?;

    // Account initialization
    let mut account_data = ZklAccount::unpack_from_slice(&zkl_account.data.borrow())?;
    if account_data.is_initialized {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Data update
    account_data.is_initialized = true;
    account_data.ec_pubkey = input_data.ec_pubkey;
    account_data.zkl_sig = input_data.zkl_sig;
    account_data.solana_sig = input_data.solana_sig;
    account_data.index = input_data.index;

    ZklAccount::pack_into_slice(&account_data, &mut zkl_account.data.borrow_mut());

    // Ed25519 imza doğrulama için güncel implementasyon
    let mut instruction_data = Vec::new();
    let num_signatures: u8 = 1;
    instruction_data.push(num_signatures);
    instruction_data.push(0); // Padding byte

    let offsets = solana_program::ed25519_program::Ed25519SignatureOffsets {
        signature_offset: 2 + 14,
        signature_instruction_index: u16::MAX,
        public_key_offset: 2 + 14 + 64,
        public_key_instruction_index: u16::MAX,
        message_data_offset: 2 + 14 + 64 + 32,
        message_data_size: message_hash.len() as u16,
        message_instruction_index: u16::MAX,
    };

    instruction_data.extend_from_slice(bytemuck::bytes_of(&offsets));
    instruction_data.extend_from_slice(&input_data.solana_sig); // 64 byte
    instruction_data.extend_from_slice(payer.key.as_ref()); // 32 byte
    instruction_data.extend_from_slice(&message_hash); // 32 byte

    let ed25519_program_id = solana_program::ed25519_program::id();
    let instruction = solana_program::instruction::Instruction {
        program_id: ed25519_program_id,
        accounts: vec![],
        data: instruction_data,
    };

    solana_program::program::invoke(
        &instruction,
        &[payer.clone(), zkl_account.clone()],
    )?;

    Ok(())
}