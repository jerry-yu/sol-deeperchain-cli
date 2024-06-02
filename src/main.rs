use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// pub struct UserAccount {
//     pub credit: u32,
//     pub reserve: [u8; 32],
// }

#[derive(BorshSerialize, BorshDeserialize, Debug)]

pub struct UserAccount {
    pub credit: u32,
    pub history: Vec<(i64, u8)>,
}

fn main() {
    let client = RpcClient::new("http://127.0.0.1:8899".to_string());
    
    let sk = vec![99,145,152,156,220,165,60,157,4,136,243,62,204,38,190,94,82,201,134,153,100,230,34,243,31,21,168,253,209,59,204,192,205,65,234,255,47,147,140,84,234,176,144
    ,169,157,219,170,161,56,150,167,230,253,232,139,25,60,121,62,74,230,112,206,32];
    let payer = Keypair::from_bytes(&sk).unwrap();

    let program_id = Pubkey::from_str("HXM5ahXEvXRrZcGRfhQ5STqaopiGL1cMTYncEtUdxdTZ").unwrap();

    let user_account = Keypair::new();

    // Initialize PDA with credit value
    let (pda, _bump_seed) = 
        Pubkey::find_program_address(&[b"user", user_account.pubkey().as_ref()], &program_id);
    let credit_value: u32 = 100;
    //let value: u32 = 200;
    //let instruction_data = [0u8].iter().chain(user_account.pubkey().as_ref()).chain(&credit_value.to_le_bytes()).cloned().collect::<Vec<u8>>();

    let instruction_data = [0u8]
        .iter()
        .chain(user_account.pubkey().as_ref())
        .chain(&credit_value.to_le_bytes())
        .cloned()
        .collect::<Vec<u8>>();

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(pda, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let instruction = Instruction::new_with_bytes(program_id, &instruction_data, accounts);
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();

    println!("Initialized PDA with credit value: {}", credit_value);

    // Update credit value
    let new_credit_value: u32 = 222;
    //let new_value: u32 = 3000;
    let instruction_data = [1u8]
        .iter()
        .chain(user_account.pubkey().as_ref())
        .chain(&new_credit_value.to_le_bytes())
        //.chain(&new_value.to_le_bytes())
        .cloned()
        .collect::<Vec<u8>>();

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        // AccountMeta::new(user_account.pubkey(), true),
        AccountMeta::new(pda, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let instruction = Instruction::new_with_bytes(program_id, &instruction_data, accounts);
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();

    println!("Updated PDA credit value to: {}", new_credit_value);

    //query
    let data = client.get_account_data(&pda).unwrap();
    let bytes: [u8; 4] = [data[3], data[2], data[1], data[0]];

    // 将字节数组转换为 u32
    let num = u32::from_le_bytes(bytes);
    let content = UserAccount::try_from_slice(&data[4..4+num as usize]).unwrap();

    println!("Query PDA credit value: {:?}", content);
}
