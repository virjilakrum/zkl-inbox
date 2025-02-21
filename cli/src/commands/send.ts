import { postVaaSolana } from '@certusone/wormhole-sdk';
import { create } from 'ipfs-http-client';
import * as zkCrypto from '@zklx/crypto';
import { Connection, PublicKey, Keypair, Transaction, TransactionInstruction, SystemProgram, sendAndConfirmTransaction } from '@solana/web3.js';
import { readFileSync } from 'fs';
import { Buffer } from 'buffer';
import { 
  KEY_REGISTRY_PROGRAM_ID,
  INBOX_PROGRAM_ID
} from '../constants';
import * as borsh from 'borsh';
const { eciesEncrypt } = require('@zklx/crypto');

// Ortam değişkenleri
const SOLANA_DEVNET_URL = process.env.SOLANA_DEVNET_URL || 'https://api.devnet.solana.com';
const WORMHOLE_CHAIN_ID = {
  solana: 1,
  ethereum: 2
} as const;

interface CurrentUser {
  ecPubkey: Uint8Array;
  solanaKeypair: Keypair;
}

// 2. FileTxRecord şemasını tanımlayalım
class FileTxRecord {
  constructor(props: {
    sender_ec_pubkey: Uint8Array,
    encrypted_link: string,
    ephemeral_pubkey: Uint8Array,
    timestamp: number
  }) {
    Object.assign(this, props);
  }
}

const FileTxRecordSchema = {
  struct: {
    sender_ec_pubkey: { array: { type: 'u8', len: 33 } },
    encrypted_link: 'string',
    ephemeral_pubkey: { array: { type: 'u8', len: 32 } },
    timestamp: 'u64'
  }
} as const;

export async function sendFile(
  filePath: string,
  recipient: string,
  chainFrom: keyof typeof WORMHOLE_CHAIN_ID,
  currentUser: CurrentUser
) {
  // 1. Dosyayı oku
  const fileBuffer = readFileSync(filePath);
  
  // 2. Alıcının EC pubkey'ini al
  const recipientPubkey = await getRecipientKey(recipient);
  
  // 3. ECIES ile şifrele
  const { encryptedData, ephemeralPubkey } = eciesEncrypt(
    Buffer.from(fileBuffer),
    recipientPubkey
  );

  // 4. IPFS'e yükle
  const ipfs = create({ 
    host: process.env.IPFS_HOST || 'localhost',
    port: Number(process.env.IPFS_PORT) || 5001
  });
  
  const { cid } = await ipfs.add(encryptedData);
  await ipfs.pin.add(cid);

  // 5. FileTxRecord oluştur
  const record = {
    sender_ec_pubkey: currentUser.ecPubkey,
    encrypted_link: cid.toString(),
    ephemeral_pubkey: ephemeralPubkey,
    timestamp: Date.now(),
  };

  // 6. Wormhole ile gönder
  const connection = new Connection(SOLANA_DEVNET_URL);
  const payer = currentUser.solanaKeypair;

  await postVaaSolana(
    connection,
    async (transaction) => {
      transaction.partialSign(payer);
      return transaction;
    },
    payer.publicKey,
    Buffer.from(JSON.stringify(record)),
    Buffer.from([WORMHOLE_CHAIN_ID[chainFrom]])
  );

  // 7. Inbox programına kayıt ekle (connection yeniden tanımlanmadan)
  const [inboxPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from('inbox'), new PublicKey(recipient).toBuffer()],
    INBOX_PROGRAM_ID
  );

  const transaction = new Transaction();
  transaction.add(
    new TransactionInstruction({
      keys: [
        { pubkey: inboxPDA, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: INBOX_PROGRAM_ID,
      data: Buffer.from(borsh.serialize(
        FileTxRecordSchema,
        new FileTxRecord(record)
      ))
    })
  );

  await sendAndConfirmTransaction(connection, transaction, [payer]);
}

async function getRecipientKey(recipientAddress: string): Promise<Uint8Array> {
  const connection = new Connection(SOLANA_DEVNET_URL);
  const [pda] = await PublicKey.findProgramAddressSync(
    [Buffer.from('zkl_account'), new PublicKey(recipientAddress).toBuffer()],
    KEY_REGISTRY_PROGRAM_ID
  );
  
  const accountInfo = await connection.getAccountInfo(pda);
  if (!accountInfo?.data) throw new Error('PDA account not found');
  return new Uint8Array(accountInfo.data.subarray(1, 34));
}       