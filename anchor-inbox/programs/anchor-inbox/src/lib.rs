use anchor_lang::prelude::*;

declare_id!("CVphDNHfy46bkV3zf3jajy9HF48osy2j7NsbA4aWxwFw");

#[program]
pub mod zkl_inbox {
    use super::*;

    /// Inbox'ı başlatır ve belirtilen alıcıya bağlar.
    pub fn initialize_inbox(
        ctx: Context<InitializeInbox>,
        recipient_ec_pubkey: [u8; 33],
        index: u32, // PDA seed'inde client-side'da kullanılacak
    ) -> Result<()> {
        let inbox_account = &mut ctx.accounts.inbox_account;
        inbox_account.recipient_ec_pubkey = recipient_ec_pubkey;
        inbox_account.recipient_wallet = ctx.accounts.user.key();
        inbox_account.messages = Vec::new();
        inbox_account.bump = *ctx.bumps.get("inbox_account").unwrap();
        Ok(())
    }

    /// Inbox'a yeni bir mesaj ekler (maksimum 100 mesaj sınırı ile).
    pub fn add_message(
        ctx: Context<AddMessage>,
        sender_ec_pubkey: [u8; 33],
        encrypted_link: String,
        ephemeral_pubkey: [u8; 33],
        timestamp: i64,
        signature: [u8; 64],
    ) -> Result<()> {
        let inbox_account = &mut ctx.accounts.inbox_account;
        if inbox_account.messages.len() >= 100 {
            return Err(error!(ErrorCode::InboxFull));
        }
        let message = FileTxRecord {
            sender_ec_pubkey,
            encrypted_link,
            ephemeral_pubkey,
            timestamp,
            signature,
        };
        inbox_account.messages.push(message);
        Ok(())
    }

    /// Inbox'taki mesajları timestamp'e göre sıralı olarak döndürür.
    pub fn get_messages(ctx: Context<GetMessages>) -> Result<Vec<FileTxRecord>> {
        let inbox_account = &ctx.accounts.inbox_account;
        let mut messages = inbox_account.messages.clone();
        messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(messages)
    }
}

#[derive(Accounts)]
#[instruction(index: u32)]
pub struct InitializeInbox<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 33 + 32 + 4 + 1, // discriminator + recipient_ec_pubkey + recipient_wallet + messages (boş vektör) + bump
        seeds = [user.key.as_ref(), &index.to_le_bytes()], // index client-side'da sağlanacak
        bump
    )]
    pub inbox_account: Account<'info, InboxAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddMessage<'info> {
    #[account(mut)]
    pub inbox_account: Account<'info, InboxAccount>,
    pub sender: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetMessages<'info> {
    pub inbox_account: Account<'info, InboxAccount>,
}

#[account]
pub struct InboxAccount {
    pub recipient_ec_pubkey: [u8; 33], // Alıcının EC public key'i
    pub recipient_wallet: Pubkey,      // Alıcının Solana cüzdan adresi
    pub messages: Vec<FileTxRecord>,   // Mesaj listesi
    pub bump: u8,                      // PDA bump değeri
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FileTxRecord {
    pub sender_ec_pubkey: [u8; 33],  // Göndericinin EC public key'i
    pub encrypted_link: String,      // Şifrelenmiş dosya linki
    pub ephemeral_pubkey: [u8; 33],  // Ephemeral public key
    pub timestamp: i64,              // Mesajın zaman damgası
    pub signature: [u8; 64],         // Mesaj imzası
}

#[error_code]
pub enum ErrorCode {
    #[msg("Inbox is full")]
    InboxFull,
}