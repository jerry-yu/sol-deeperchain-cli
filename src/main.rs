use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use sol_deeperchain::{
    instruction::CreditInstruction,
    state::{CreditSetting, CreditSettings, TokenAccount, UserAccount},
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    compute_budget
};
use std::str::FromStr;

fn main() {
    let client = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".to_string(),
        CommitmentConfig::processed(),
    );

    let sk = vec![
        99, 145, 152, 156, 220, 165, 60, 157, 4, 136, 243, 62, 204, 38, 190, 94, 82, 201, 134, 153,
        100, 230, 34, 243, 31, 21, 168, 253, 209, 59, 204, 192, 205, 65, 234, 255, 47, 147, 140,
        84, 234, 176, 144, 169, 157, 219, 170, 161, 56, 150, 167, 230, 253, 232, 139, 25, 60, 121,
        62, 74, 230, 112, 206, 32,
    ];
    let payer = Keypair::from_bytes(&sk).unwrap();

    let program_id = Pubkey::from_str("HXM5ahXEvXRrZcGRfhQ5STqaopiGL1cMTYncEtUdxdTZ").unwrap();

    // init credit setting config and token address
    let (init_pda, _bump_seed) = Pubkey::find_program_address(&[b"credit_setting"], &program_id);
    let (init_token, _bump_seed) = Pubkey::find_program_address(&[b"dpr_token"], &program_id);
    let (mint_authority, _bump_seed) = Pubkey::find_program_address(&[b"mint_authority"], &program_id);
    
    println!("mint_authority {:?}",mint_authority);

    let campaign_id: u16 = 1;
    let settings = CreditSettings {
        settings: vec![
            CreditSetting {
                campaign_id,
                level: 1,
                daily_reward: 1,
            },
            CreditSetting {
                campaign_id,
                level: 2,
                daily_reward: 2,
            },
            CreditSetting {
                campaign_id,
                level: 3,
                daily_reward: 3,
            },
        ],
    };

    //DPR token address
    let token = TokenAccount {
        token: Pubkey::from_str("5kSfsEoPXv4cgKx4Ct2irz9xF6mWcTo1NLFfKfKs11fu").unwrap(),
    };

    let init_instruction = CreditInstruction::Init { settings, token };

    let instruction_data = to_vec(&init_instruction).unwrap();

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(init_pda, false),
        AccountMeta::new(init_token, false),
        AccountMeta::new(mint_authority, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let instruction = Instruction::new_with_bytes(program_id, &instruction_data, accounts);
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();

    //let dst_user = Pubkey::from_str("5kSfsEoPXv4cgKx4Ct2irz9xF6mWcTo1NLFfKfKs11fu").unwrap();

    let dst_user = payer.pubkey();
    // add credit record for users
    let (user_credit, _bump_seed) =
        Pubkey::find_program_address(&[b"user", dst_user.as_ref()], &program_id);
    let credit_value: i32 = 199;

    let add_instruction = CreditInstruction::Add {
        pk: dst_user,
        credit: credit_value,
        campaign: campaign_id,
        reward_since: 1,
    };
    let instruction_data = to_vec(&add_instruction).unwrap();

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(user_credit, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let instruction = Instruction::new_with_bytes(program_id, &instruction_data, accounts);
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();

    println!("Add PDA with credit value: {}", credit_value);

    // set token address
    let new_token_address =
        Pubkey::from_str("9j2VGAtkbW3TyLLrWqh7irj7ZFHHTvKqaYNFNCTW4ybh").unwrap();

    let token_instruction = CreditInstruction::SetTokenAddress {
        address: new_token_address,
    };
    let instruction_data = to_vec(&token_instruction).unwrap();

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(init_token, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let instruction = Instruction::new_with_bytes(program_id, &instruction_data, accounts);
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();

    //query token address
    let data = client.get_account_data(&init_token).unwrap();
    let content = TokenAccount::try_from_slice(&data).unwrap();

    println!("Query PDA TokenAccount : {:?}", content);

    //claim token
    let claim_instruction = CreditInstruction::Claim;
    let instruction_data = to_vec(&claim_instruction).unwrap();

    let user_token_address = spl_associated_token_account::get_associated_token_address(
        &payer.pubkey(),
        &new_token_address,
    );

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(mint_authority, false),
        
        AccountMeta::new(user_credit, false),
        AccountMeta::new(user_token_address, false),
        AccountMeta::new(init_token, false),
        AccountMeta::new(new_token_address, false),
        AccountMeta::new(init_pda, false),

        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(program_id, false),
    ];

    let compute_budget_instruction = compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_400_000);

    let instruction = Instruction::new_with_bytes(program_id, &instruction_data, accounts);
    let mut transaction = Transaction::new_with_payer(&[compute_budget_instruction,instruction], Some(&payer.pubkey()));
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();

    //query credit value
    let data = client.get_account_data(&user_credit).unwrap();
    let bytes: [u8; 4] = [data[3], data[2], data[1], data[0]];

    let num = u32::from_le_bytes(bytes);
    let content = UserAccount::try_from_slice(&data[4..4 + num as usize]).unwrap();

    println!("Query PDA credit value: {:?} num {}", content, num);

    //query credit setting
    let data = client.get_account_data(&init_pda).unwrap();
    let content = CreditSettings::try_from_slice(&data).unwrap();
    println!("Query PDA credit settings: {:?}", content);
}
