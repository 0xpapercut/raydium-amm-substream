use bs58;

// use pb::system_program;
use substreams::errors::Error;
use substreams_solana::pb::sf::solana::r#type::v1::ConfirmedTransaction;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

use substreams_solana_utils as utils;
use utils::transaction::{get_context, TransactionContext};
use utils::instruction::{get_structured_instructions, StructuredInstructions, StructuredInstruction};

pub mod pubkey;
pub use pubkey::Pubkey;

pub mod system_program;
use system_program::SystemInstruction;
use system_program::SYSTEM_PROGRAM_ID;

pub mod pb;
use pb::system_program::{
    SystemProgramBlockEvents,
    SystemProgramTransactionEvents,
    SystemProgramEvent,
    CreateAccountEvent,
    AssignEvent,
    TransferEvent,
    CreateAccountWithSeedEvent,
    AdvanceNonceAccountEvent,
    WithdrawNonceAccountEvent,
    InitializeNonceAccountEvent,
    AuthorizeNonceAccountEvent,
    AllocateEvent,
    AllocateWithSeedEvent,
    AssignWithSeedEvent,
    TransferWithSeedEvent,
    UpgradeNonceAccountEvent,
};
use pb::system_program::system_program_event::Event;

#[substreams::handlers::map]
fn system_program_events(block: Block) -> Result<SystemProgramBlockEvents, Error> {
    let transactions = parse_block(&block);
    Ok(SystemProgramBlockEvents { transactions })
}

pub fn parse_block(block: &Block) -> Vec<SystemProgramTransactionEvents> {
    let mut block_events: Vec<SystemProgramTransactionEvents> = Vec::new();
    for (i, transaction) in block.transactions.iter().enumerate() {
        if let Ok(events) = parse_transaction(transaction) {
            if !events.is_empty() {
                block_events.push(SystemProgramTransactionEvents {
                    signature: utils::transaction::get_signature(transaction),
                    transaction_index: i as u32,
                    events,
                });
            }
        }
    }
    block_events
}

pub fn parse_transaction(transaction: &ConfirmedTransaction) -> Result<Vec<SystemProgramEvent>, String> {
    if let Some(_) = transaction.meta.as_ref().unwrap().err {
        return Err("Cannot parse failed transaction.".to_string());
    }

    let mut events: Vec<SystemProgramEvent> = Vec::new();

    let context = get_context(transaction);
    let instructions = get_structured_instructions(transaction)?;

    for (i, instruction) in instructions.flattened().iter().enumerate() {
        if bs58::encode(context.get_account_from_index(instruction.program_id_index() as usize)).into_string() == SYSTEM_PROGRAM_ID {
            match parse_instruction(instruction, &context) {
                Ok(event) => {
                    events.push(SystemProgramEvent { instruction_index: i as u32, event });
                },
                Err(e) => substreams::log::println(e),
            }
        }
    }


    Ok(events)
}

pub fn parse_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext
) -> Result<Option<Event>, String> {
    if bs58::encode(context.get_account_from_index(instruction.program_id_index() as usize)).into_string() != SYSTEM_PROGRAM_ID {
        return Err("Not a System Program instruction.".to_string());
    }
    let unpacked = SystemInstruction::unpack(&instruction.data())?;
    match unpacked {
        SystemInstruction::CreateAccount(create_account) => {
            _parse_create_account_instruction(instruction, context, &create_account).map(|x| Some(Event::CreateAccount(x)))
        },
        SystemInstruction::Assign(assign) => {
            _parse_assign_instruction(instruction, context, &assign).map(|x| Some(Event::Assign(x)))
        },
        SystemInstruction::Transfer(transfer) => {
            _parse_transfer_instruction(instruction, context, &transfer).map(|x| Some(Event::Transfer(x)))
        },
        SystemInstruction::CreateAccountWithSeed(create_account_with_seed) => {
            _parse_create_account_with_seed_instruction(instruction, context, &create_account_with_seed).map(|x| Some(Event::CreateAccountWithSeed(x)))
        },
        SystemInstruction::AdvanceNonceAccount => {
            _parse_advance_nonce_account_instruction(instruction, context).map(|x| Some(Event::AdvanceNonceAccount(x)))
        },
        SystemInstruction::WithdrawNonceAccount(lamports) => {
            _parse_withdraw_nonce_account_instruction(instruction, context, lamports).map(|x| Some(Event::WithdrawNonceAccount(x)))
        },
        SystemInstruction::InitializeNonceAccount(pubkey) => {
            _parse_initialize_nonce_account_instruction(instruction, context, pubkey).map(|x| Some(Event::InitializeNonceAccount(x)))
        },
        SystemInstruction::AuthorizeNonceAccount(pubkey) => {
            _parse_authorize_nonce_account_instruction(instruction, context, pubkey).map(|x| Some(Event::AuthorizeNonceAccount(x)))
        },
        SystemInstruction::Allocate(allocate) => {
            _parse_allocate_instruction(instruction, context, &allocate).map(|x| Some(Event::Allocate(x)))
        },
        SystemInstruction::AllocateWithSeed(allocate_with_seed) => {
            _parse_allocate_with_seed_instruction(instruction, context, &allocate_with_seed).map(|x| Some(Event::AllocateWithSeed(x)))
        },
        SystemInstruction::AssignWithSeed(assign_with_seed) => {
            _parse_assign_with_seed_instruction(instruction, context, &assign_with_seed).map(|x| Some(Event::AssignWithSeed(x)))
        },
        SystemInstruction::TransferWithSeed(transfer_with_seed) => {
            _parse_transfer_with_seed_instruction(instruction, context, transfer_with_seed).map(|x| Some(Event::TransferWithSeed(x)))
        },
        SystemInstruction::UpgradeNonceAccount => {
            _parse_upgrade_nonce_account_instruction(instruction, context).map(|x| Some(Event::UpgradeNonceAccount(x)))
        }
    }
}

fn _parse_create_account_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    create_account: &system_program::CreateAccount,
) -> Result<CreateAccountEvent, String> {
    let funding_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let new_account = bs58::encode(context.get_account_from_index(instruction.accounts()[1] as usize)).into_string();
    let lamports = create_account.lamports;
    let owner = bs58::encode(create_account.owner.0).into_string();
    let space = create_account.space;

    Ok(CreateAccountEvent {
        funding_account,
        new_account,
        lamports,
        owner,
        space,
    })
}

fn _parse_assign_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    assign: &system_program::Assign,
) -> Result<AssignEvent, String> {
    let assigned_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let owner = bs58::encode(assign.owner.0).into_string();

    Ok(AssignEvent {
        assigned_account,
        owner,
    })
}

fn _parse_transfer_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    transfer: &system_program::Transfer,
) -> Result<TransferEvent, String> {
    let funding_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let recipient_account = bs58::encode(context.get_account_from_index(instruction.accounts()[1] as usize)).into_string();
    let lamports = transfer.lamports;

    Ok(TransferEvent {
        funding_account,
        recipient_account,
        lamports,
    })
}

fn _parse_create_account_with_seed_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    create_account_with_seed: &system_program::CreateAccountWithSeed,
) -> Result<CreateAccountWithSeedEvent, String> {
    let funding_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let created_account = bs58::encode(context.get_account_from_index(instruction.accounts()[1] as usize)).into_string();
    let base_account = bs58::encode(create_account_with_seed.base.0).into_string();
    let lamports = create_account_with_seed.lamports;
    let owner = bs58::encode(create_account_with_seed.owner.0).into_string();
    let seed = create_account_with_seed.seed.0.clone();
    let space = create_account_with_seed.space;

    Ok(CreateAccountWithSeedEvent {
        funding_account,
        created_account,
        base_account,
        seed,
        lamports,
        space,
        owner,
    })
}

fn _parse_advance_nonce_account_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
) -> Result<AdvanceNonceAccountEvent, String> {
    let nonce_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let nonce_authority = bs58::encode(context.get_account_from_index(instruction.accounts()[2] as usize)).into_string();

    Ok(AdvanceNonceAccountEvent {
        nonce_account,
        nonce_authority,
    })
}

fn _parse_withdraw_nonce_account_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    lamports: u64,
) -> Result<WithdrawNonceAccountEvent, String> {
    let nonce_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let recipient_account = bs58::encode(context.get_account_from_index(instruction.accounts()[1] as usize)).into_string();
    let nonce_authority = bs58::encode(context.get_account_from_index(instruction.accounts()[4] as usize)).into_string();

    Ok(WithdrawNonceAccountEvent {
        nonce_account,
        recipient_account,
        nonce_authority,
        lamports,
    })
}

fn _parse_initialize_nonce_account_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    authority: Pubkey,
) -> Result<InitializeNonceAccountEvent, String> {
    let nonce_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let nonce_authority = bs58::encode(authority.0).into_string();

    Ok(InitializeNonceAccountEvent {
        nonce_account,
        nonce_authority,
    })
}

fn _parse_authorize_nonce_account_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    pubkey: Pubkey,
) -> Result<AuthorizeNonceAccountEvent, String> {
    let nonce_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let nonce_authority = bs58::encode(context.get_account_from_index(instruction.accounts()[1] as usize)).into_string();
    let new_nonce_authority = bs58::encode(pubkey.0).into_string();

    Ok(AuthorizeNonceAccountEvent {
        nonce_account,
        nonce_authority,
        new_nonce_authority,
    })
}

fn _parse_allocate_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    allocate: &system_program::Allocate,
) -> Result<AllocateEvent, String> {
    let account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let space = allocate.space;

    Ok(AllocateEvent {
        account,
        space,
    })
}

fn _parse_allocate_with_seed_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    allocate_with_seed: &system_program::AllocateWithSeed,
) -> Result<AllocateWithSeedEvent, String> {
    let allocated_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let space = allocate_with_seed.space;
    let base_account = bs58::encode(allocate_with_seed.base.0).into_string();
    let owner = bs58::encode(allocate_with_seed.owner.0).into_string();
    let seed = bs58::encode(&allocate_with_seed.seed.0).into_string();

    Ok(AllocateWithSeedEvent {
        allocated_account,
        base_account,
        seed,
        owner,
        space,
    })
}

fn _parse_assign_with_seed_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    assign_with_seed: &system_program::AssignWithSeed,
) -> Result<AssignWithSeedEvent, String> {
    let assigned_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let base_account = bs58::encode(assign_with_seed.base.0).into_string();
    let owner = bs58::encode(assign_with_seed.owner.0).into_string();
    let seed = bs58::encode(&assign_with_seed.seed.0).into_string();
    Ok(AssignWithSeedEvent {
        assigned_account,
        base_account,
        owner,
        seed,
    })
}

fn _parse_transfer_with_seed_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
    transfer_with_seed: system_program::TransferWithSeed
) -> Result<TransferWithSeedEvent, String> {
    let funding_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();
    let base_account = bs58::encode(context.get_account_from_index(instruction.accounts()[1] as usize)).into_string();
    let recipient_account = bs58::encode(context.get_account_from_index(instruction.accounts()[2] as usize)).into_string();
    let from_owner = bs58::encode(transfer_with_seed.from_owner.0).into_string();
    let from_seed = bs58::encode(transfer_with_seed.from_seed.0).into_string();
    let lamports = transfer_with_seed.lamports;

    Ok(TransferWithSeedEvent {
        funding_account,
        base_account,
        recipient_account,
        from_owner,
        from_seed,
        lamports,
    })
}

fn _parse_upgrade_nonce_account_instruction(
    instruction: &StructuredInstruction,
    context: &TransactionContext,
) -> Result<UpgradeNonceAccountEvent, String> {
    let nonce_account = bs58::encode(context.get_account_from_index(instruction.accounts()[0] as usize)).into_string();

    Ok(UpgradeNonceAccountEvent {
        nonce_account,
    })
}
