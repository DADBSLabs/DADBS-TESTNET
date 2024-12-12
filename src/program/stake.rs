use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    program::{invoke, invoke_signed},
    sysvar::{rent::Rent, Sysvar},
};

use borsh::{BorshDeserialize, BorshSerialize};

// 定义质押账户的数据结构
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StakeAccount {
    pub owner: Pubkey,           // 质押者的公钥
    pub amount: u64,             // 质押金额
    pub locked_until: i64,       // 锁定期
    pub is_active: bool,         // 是否激活
}

// 定义质押指令
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum StakeInstruction {
    // 创建质押账户并质押SOL
    CreateStake {
        amount: u64,
        lock_period: i64,
    },
    // 取回质押的SOL
    Withdraw {
        amount: u64,
    },
}

// 程序入口点
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StakeInstruction::try_from_slice(instruction_data)?;
    
    match instruction {
        StakeInstruction::CreateStake { amount, lock_period } => {
            process_create_stake(program_id, accounts, amount, lock_period)
        }
        StakeInstruction::Withdraw { amount } => {
            process_withdraw(program_id, accounts, amount)
        }
    }
}

// 处理质押创建
fn process_create_stake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    lock_period: i64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // 获取相关账户
    let staker_account = next_account_info(account_info_iter)?;    // 质押者账户
    let stake_account = next_account_info(account_info_iter)?;     // 质押存储账户
    let system_program = next_account_info(account_info_iter)?;    // 系统程序
    
    // 验证质押金额（最少10 SOL）
    if amount < 10_000_000_000 {
        return Err(ProgramError::InvalidArgument);
    }

    // 创建质押账户并转移SOL
    let rent = Rent::get()?;
    let stake_account_data = StakeAccount {
        owner: *staker_account.key,
        amount,
        locked_until: lock_period,
        is_active: true,
    };

    // 计算所需空间
    let space = stake_account_data.try_to_vec()?.len();
    let rent_lamports = rent.minimum_balance(space);

    // 创建账户
    invoke(
        &system_instruction::create_account(
            staker_account.key,
            stake_account.key,
            amount + rent_lamports,
            space as u64,
            program_id,
        ),
        &[
            staker_account.clone(),
            stake_account.clone(),
            system_program.clone(),
        ],
    )?;

    // 保存质押信息
    stake_account_data.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;

    msg!("Stake account created and SOL locked successfully");
    Ok(())
}

// 处理提取质押
fn process_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let staker_account = next_account_info(account_info_iter)?;
    let stake_account = next_account_info(account_info_iter)?;
    
    // 验证账户所有权
    if stake_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // 读取质押账户数据
    let mut stake_data = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
    
    // 验证提取者是否是质押者
    if stake_data.owner != *staker_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    // 检查锁定期
    if stake_data.locked_until > 0 {
        return Err(ProgramError::InvalidArgument);
    }

    // 验证提取金额
    if amount > stake_data.amount {
        return Err(ProgramError::InsufficientFunds);
    }

    // 转移SOL回质押者账户
    **stake_account.try_borrow_mut_lamports()? -= amount;
    **staker_account.try_borrow_mut_lamports()? += amount;

    // 更新质押账户数据
    stake_data.amount -= amount;
    stake_data.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;

    msg!("Withdrew {} lamports from stake account", amount);
    Ok(())
}
