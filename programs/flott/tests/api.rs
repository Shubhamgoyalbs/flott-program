use anchor_lang::{InstructionData, ToAccountMetas, system_program::ID as SystemProgramId, AccountDeserialize};
use solana_transaction::Transaction;
use {
  litesvm::LiteSVM,
  solana_instruction::{Instruction},
  solana_keypair::Keypair,
  solana_pubkey::{Pubkey},
  solana_message::{Message},
  solana_signer::Signer,
};
use b58;

#[test]
pub fn api_instructions() {
  let mut svm = LiteSVM::new();
  let program_id = flott::id();
  let api_key_holder = Keypair::new();
  let bytes = include_bytes!("../../../target/deploy/flott.so");
  svm.add_program(program_id, bytes).expect("Failed to read server_test.json");
  svm.airdrop(&api_key_holder.pubkey(), 1_000_000_000).unwrap();

  let server_auth = Keypair::from_base58_string(&b58::encode(&[25,135,148,227,232,27,155,43,163,97,35,40,187,181,153,216,196,51,97,162,123,91,103,56,156,58,46,13,101,73,210,242,165,72,16,64,89,122,86,191,53,230,224,226,130,113,36,212,172,237,92,71,127,157,165,26,37,238,80,115,75,146,25,100]).to_string());
  let api_mpc = Keypair::new();

  let api_fee_per: u32 = 1000000;
  let (api_user_pda, _api_user_bump) = Pubkey::find_program_address(
    &[
      b"api",
      b"user",
      api_key_holder.pubkey().as_ref(),
    ],
    &program_id,
  );
  
  let (event_authority, _event_authority_bump) = Pubkey::find_program_address(
    &[b"__event_authority"],
    &program_id
  );

  let (vault_pda, _vault_bump) = Pubkey::find_program_address(
    &[
      b"api",
      b"user",
      b"vault",
      api_user_pda.as_ref(),
    ],
    &program_id,
  );
  
  let initialize_api_user_ix = Instruction::new_with_bytes(
    program_id,
    &flott::instruction::InitializeApiUser {
      fee_percentage: api_fee_per
    }.data(),
    flott::accounts::InitializeApiUser {
      owner: api_key_holder.pubkey(),
      api_user: api_user_pda,
      vault: vault_pda,
      system_program: SystemProgramId
    }.to_account_metas(None)
  );
  
  let authorize_api_user_ix = Instruction::new_with_bytes(
    program_id,
    &flott::instruction::AuthorizeApiUser {}.data(),
    flott::accounts::AuthorizeApiUser {
      server: server_auth.pubkey(),
      authority: api_mpc.pubkey(),
      owner: api_key_holder.pubkey(),
      vault: vault_pda,
      api_user: api_user_pda,
      event_authority,
      system_program: SystemProgramId,
      program: program_id,
    }.to_account_metas(None)
  );
  
  let deactivate_api_user_ix = Instruction::new_with_bytes(
    program_id,
    &flott::instruction::DeactivateApiUser {}.data(),
    flott::accounts::DeactivateApiUser {
      authority: api_mpc.pubkey(),
      owner: api_key_holder.pubkey(),
      api_user: api_user_pda,
      event_authority,
      system_program: SystemProgramId,
      program: program_id,
    }.to_account_metas(None)
  );
  
  let activate_api_user_ix = Instruction::new_with_bytes(
    program_id,
    &flott::instruction::ActivateApiUser {}.data(),
    flott::accounts::ActivateApiUser {
      authority: api_mpc.pubkey(),
      owner: api_key_holder.pubkey(),
      vault: vault_pda,
      api_user: api_user_pda,
      event_authority,
      system_program: SystemProgramId,
      program: program_id,
    }.to_account_metas(None)
  );
  
  let deposit_api_user_ix = Instruction::new_with_bytes(
    program_id,
    &flott::instruction::DepositToVault { amount: 2_000_000 }.data(),
    flott::accounts::DepositToVault {
      owner: api_key_holder.pubkey(),
      authority: api_mpc.pubkey(),
      vault: vault_pda,
      api_user: api_user_pda,
      system_program: SystemProgramId,
    }.to_account_metas(None)
  );
  
  let withdraw_api_user_ix = Instruction::new_with_bytes(
    program_id,
    &flott::instruction::WithdrawFromVault { amount: 1_000_000 }.data(),
    flott::accounts::WithdrawFromVault {
      owner: api_key_holder.pubkey(),
      authority: api_mpc.pubkey(),
      vault: vault_pda,
      api_user: api_user_pda,
      system_program: SystemProgramId,
    }.to_account_metas(None)
  );
  
  let transaction = Transaction::new(&[&api_key_holder, &server_auth, &api_mpc], Message::new(&[
    initialize_api_user_ix,
    authorize_api_user_ix,
    deactivate_api_user_ix,
    activate_api_user_ix,
    deposit_api_user_ix,
    withdraw_api_user_ix
  ], Some(&api_key_holder.pubkey())), svm.latest_blockhash());
  svm.send_transaction(transaction).unwrap();
  
  let api_user_raw = svm.get_account(&api_user_pda).unwrap();
  let api_user_data = flott::state::ApiUser::try_deserialize(&mut api_user_raw.data.as_ref()).unwrap();
  
  assert_eq!(api_user_data.is_active, true);
}