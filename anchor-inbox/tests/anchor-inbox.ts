import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { ZklInbox } from "../target/types/zkl_inbox"; // IDL dosyasından türetilmiş türleri içe aktar

describe("zkl-inbox", () => {
  // Provider ve program ayarları
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.ZklInbox as Program<ZklInbox>;
  const user = provider.wallet as anchor.Wallet;

  // Örnek bir 33 byte EC public key (test için sabit bir değer)
  const recipientEcPubkey = Buffer.alloc(33, 1); // 33 baytlık bir buffer, test için 1 ile doldurulmuş

  it("Initialize Inbox", async () => {
    // Index değeri (örnek olarak 0 kullanıyoruz)
    const index = 0;

    // PDA'yı client-side'da oluştur
    const [inboxPda, bump] = await PublicKey.findProgramAddress(
      [
        user.publicKey.toBuffer(),              // Kullanıcının Solana public key'i
        Buffer.from(new anchor.BN(index).toArray("le", 4)), // index'i little-endian 4 byte olarak dönüştür
      ],
      program.programId
    );

    // initialize_inbox fonksiyonunu çağır
    await program.methods
      .initializeInbox(recipientEcPubkey, index)
      .accounts({
        inboxAccount: inboxPda,              // PDA adresi
        user: user.publicKey,               // Kullanıcının public key'i
        systemProgram: anchor.web3.SystemProgram.programId, // SystemProgram
      })
      .signers([]) // user zaten provider.wallet tarafından sağlanıyor
      .rpc();

    // Inbox'ın başarıyla oluşturulduğunu doğrula (opsiyonel)
    const inboxAccount = await program.account.inboxAccount.fetch(inboxPda);
    console.log("Inbox initialized:", {
      recipientEcPubkey: inboxAccount.recipientEcPubkey,
      recipientWallet: inboxAccount.recipientWallet.toBase58(),
      messages: inboxAccount.messages,
      bump: inboxAccount.bump,
    });
  });
});