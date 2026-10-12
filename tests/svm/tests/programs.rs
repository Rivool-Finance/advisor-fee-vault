use anchor_lang::solana_program::{program_option::COption, program_pack::Pack};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas};
use litesvm::{types::TransactionResult, LiteSVM};
use solana_account::Account;
use solana_clock::Clock;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_loader_v3_interface::state::UpgradeableLoaderState;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};
use wincode::Serialize;

const USDC: u64 = 1_000_000;
const SCALE: u128 = stand_in_yield::PRICE_SCALE;

#[allow(clippy::result_large_err)] // Keep LiteSVM's native result and complete failure metadata in tests.
fn send(svm: &mut LiteSVM, signer: &Keypair, ix: Instruction) -> TransactionResult {
    svm.expire_blockhash();
    let tx = Transaction::new(
        &[signer],
        Message::new(&[ix], Some(&signer.pubkey())),
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx)
}

fn rejected(result: TransactionResult, code: &str) {
    let error = result.expect_err("transaction should fail");
    assert!(
        error.meta.logs.iter().any(|log| log.contains(code)),
        "{error:?}"
    );
}

fn load(svm: &mut LiteSVM, id: Pubkey, name: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy")
        .join(format!("{name}.so"));
    svm.add_program_from_file(id, path)
        .expect("run anchor build before the SVM tests");
}

fn token_account(svm: &mut LiteSVM, address: Pubkey, mint: Pubkey, owner: Pubkey, amount: u64) {
    let mut data = vec![0; TokenAccount::LEN];
    TokenAccount::pack(
        TokenAccount {
            mint,
            owner,
            amount,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        &mut data,
    )
    .unwrap();
    svm.set_account(
        address,
        Account {
            lamports: svm.minimum_balance_for_rent_exemption(data.len()),
            data,
            owner: spl_token_interface::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

fn mint_account(svm: &mut LiteSVM, address: Pubkey, authority: Pubkey, supply: u64, decimals: u8) {
    let mut data = vec![0; Mint::LEN];
    Mint::pack(
        Mint {
            mint_authority: COption::Some(authority),
            supply,
            decimals,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        &mut data,
    )
    .unwrap();
    svm.set_account(
        address,
        Account {
            lamports: svm.minimum_balance_for_rent_exemption(data.len()),
            data,
            owner: spl_token_interface::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

fn balance(svm: &LiteSVM, address: Pubkey) -> u64 {
    TokenAccount::unpack(&svm.get_account(&address).unwrap().data)
        .unwrap()
        .amount
}

fn read<T: AccountDeserialize>(svm: &LiteSVM, address: Pubkey) -> T {
    T::try_deserialize(&mut svm.get_account(&address).unwrap().data.as_slice()).unwrap()
}

struct YieldFixture {
    svm: LiteSVM,
    authority: Keypair,
    client: Keypair,
    mint: Pubkey,
    vault: Pubkey,
    shares_mint: Pubkey,
    reserve: Pubkey,
    assets: Pubkey,
    shares: Pubkey,
}

impl YieldFixture {
    fn new() -> Self {
        let mut svm = LiteSVM::new();
        load(&mut svm, stand_in_yield::ID, "stand_in_yield");
        let authority = Keypair::new();
        let client = Keypair::new();
        for signer in [&authority, &client] {
            svm.airdrop(&signer.pubkey(), 2_000_000_000).unwrap();
        }
        let mint = Pubkey::new_unique();
        mint_account(&mut svm, mint, authority.pubkey(), 1040 * USDC, 6);
        let vault = Pubkey::find_program_address(
            &[b"yield", authority.pubkey().as_ref(), mint.as_ref()],
            &stand_in_yield::ID,
        )
        .0;
        let shares_mint =
            Pubkey::find_program_address(&[b"shares", vault.as_ref()], &stand_in_yield::ID).0;
        let reserve =
            Pubkey::find_program_address(&[b"reserve", vault.as_ref()], &stand_in_yield::ID).0;
        let assets = Pubkey::new_unique();
        let shares = Pubkey::new_unique();
        token_account(&mut svm, assets, mint, client.pubkey(), 1040 * USDC);
        let ix = Instruction {
            program_id: stand_in_yield::ID,
            accounts: stand_in_yield::accounts::Initialize {
                authority: authority.pubkey(),
                mint,
                vault,
                shares_mint,
                reserve,
                token_program: spl_token_interface::ID,
                system_program: solana_system_interface::program::ID,
            }
            .to_account_metas(None),
            data: stand_in_yield::instruction::Initialize {}.data(),
        };
        send(&mut svm, &authority, ix).unwrap();
        token_account(&mut svm, shares, shares_mint, client.pubkey(), 0);
        Self {
            svm,
            authority,
            client,
            mint,
            vault,
            shares_mint,
            reserve,
            assets,
            shares,
        }
    }

    fn transact(&self, owner: Pubkey, data: Vec<u8>) -> Instruction {
        Instruction {
            program_id: stand_in_yield::ID,
            accounts: stand_in_yield::accounts::Transact {
                owner,
                vault: self.vault,
                mint: self.mint,
                shares_mint: self.shares_mint,
                reserve: self.reserve,
                owner_assets: self.assets,
                owner_shares: self.shares,
                token_program: spl_token_interface::ID,
            }
            .to_account_metas(None),
            data,
        }
    }

    fn deposit(&mut self) {
        let ix = self.transact(
            self.client.pubkey(),
            stand_in_yield::instruction::Deposit {
                assets: 1000 * USDC,
                min_shares: 1000 * USDC,
            }
            .data(),
        );
        send(&mut self.svm, &self.client, ix).unwrap();
    }

    fn rate_instruction(&self, authority: Pubkey, price: u128) -> Instruction {
        Instruction {
            program_id: stand_in_yield::ID,
            accounts: stand_in_yield::accounts::SetExchangeRate {
                authority,
                vault: self.vault,
                shares_mint: self.shares_mint,
                reserve: self.reserve,
            }
            .to_account_metas(None),
            data: stand_in_yield::instruction::SetExchangeRate { price }.data(),
        }
    }

    fn fund_yield(&mut self) {
        let ix = spl_token_interface::instruction::transfer_checked(
            &spl_token_interface::ID,
            &self.assets,
            &self.mint,
            &self.reserve,
            &self.client.pubkey(),
            &[],
            40 * USDC,
            6,
        )
        .unwrap();
        let ix = Instruction {
            program_id: ix.program_id,
            data: ix.data,
            accounts: ix
                .accounts
                .into_iter()
                .map(|m| AccountMeta {
                    pubkey: m.pubkey,
                    is_signer: m.is_signer,
                    is_writable: m.is_writable,
                })
                .collect(),
        };
        send(&mut self.svm, &self.client, ix).unwrap();
    }
}

#[test]
fn funded_yield_round_trip_executes_real_token_cpis() {
    let mut f = YieldFixture::new();
    f.deposit();
    assert_eq!(balance(&f.svm, f.reserve), 1000 * USDC);
    assert_eq!(balance(&f.svm, f.shares), 1000 * USDC);
    let mut clock = f.svm.get_sysvar::<Clock>();
    clock.unix_timestamp += 30 * 86_400;
    f.svm.set_sysvar(&clock);
    f.fund_yield();
    let ix = f.rate_instruction(f.authority.pubkey(), SCALE * 104 / 100);
    send(&mut f.svm, &f.authority, ix).unwrap();
    let state: stand_in_yield::YieldVault = read(&f.svm, f.vault);
    assert_eq!(state.price_time_index, SCALE * 30 * 86_400);
    let ix = f.transact(
        f.client.pubkey(),
        stand_in_yield::instruction::Redeem {
            shares: 1000 * USDC,
            min_assets: 1040 * USDC,
        }
        .data(),
    );
    send(&mut f.svm, &f.client, ix).unwrap();
    assert_eq!(balance(&f.svm, f.assets), 1040 * USDC);
    assert_eq!(balance(&f.svm, f.reserve), 0);
    assert_eq!(balance(&f.svm, f.shares), 0);
}

#[test]
fn unfunded_rate_increase_and_wrong_authority_are_rejected() {
    let mut f = YieldFixture::new();
    f.deposit();
    let ix = f.rate_instruction(f.authority.pubkey(), SCALE * 2);
    rejected(send(&mut f.svm, &f.authority, ix), "InsufficientBacking");
    let ix = f.rate_instruction(f.client.pubkey(), SCALE);
    rejected(send(&mut f.svm, &f.client, ix), "ConstraintHasOne");
    assert_eq!(
        read::<stand_in_yield::YieldVault>(&f.svm, f.vault).price,
        SCALE
    );
    assert_eq!(balance(&f.svm, f.reserve), 1000 * USDC);
}

#[test]
fn slippage_failure_rolls_back_balances_and_shares() {
    let mut f = YieldFixture::new();
    let ix = f.transact(
        f.client.pubkey(),
        stand_in_yield::instruction::Deposit {
            assets: 1000 * USDC,
            min_shares: 1001 * USDC,
        }
        .data(),
    );
    rejected(send(&mut f.svm, &f.client, ix), "Slippage");
    assert_eq!(balance(&f.svm, f.assets), 1040 * USDC);
    assert_eq!(balance(&f.svm, f.reserve), 0);
    f.deposit();
    let ix = f.transact(
        f.client.pubkey(),
        stand_in_yield::instruction::Redeem {
            shares: 1000 * USDC,
            min_assets: 1001 * USDC,
        }
        .data(),
    );
    rejected(send(&mut f.svm, &f.client, ix), "Slippage");
    assert_eq!(balance(&f.svm, f.shares), 1000 * USDC);
    assert_eq!(balance(&f.svm, f.reserve), 1000 * USDC);
}

#[test]
fn an_authority_cannot_redeem_a_clients_shares() {
    let mut f = YieldFixture::new();
    f.deposit();
    let ix = f.transact(
        f.authority.pubkey(),
        stand_in_yield::instruction::Redeem {
            shares: 1000 * USDC,
            min_assets: 0,
        }
        .data(),
    );
    rejected(send(&mut f.svm, &f.authority, ix), "ConstraintTokenOwner");
    assert_eq!(balance(&f.svm, f.shares), 1000 * USDC);
    assert_eq!(balance(&f.svm, f.reserve), 1000 * USDC);
}

#[test]
fn source_account_substitution_is_rejected() {
    let mut f = YieldFixture::new();
    let mut ix = f.transact(
        f.client.pubkey(),
        stand_in_yield::instruction::Deposit {
            assets: USDC,
            min_shares: USDC,
        }
        .data(),
    );
    let fake = Pubkey::new_unique();
    token_account(&mut f.svm, fake, f.mint, f.vault, 0);
    ix.accounts
        .iter_mut()
        .find(|a| a.pubkey == f.reserve)
        .unwrap()
        .pubkey = fake;
    rejected(send(&mut f.svm, &f.client, ix), "ConstraintSeeds");
    assert_eq!(balance(&f.svm, f.assets), 1040 * USDC);
}

#[test]
fn zero_price_and_backwards_clock_leave_the_vault_unchanged() {
    let mut f = YieldFixture::new();
    let ix = f.rate_instruction(f.authority.pubkey(), 0);
    rejected(send(&mut f.svm, &f.authority, ix), "InvalidPrice");
    let before: stand_in_yield::YieldVault = read(&f.svm, f.vault);
    let mut clock = f.svm.get_sysvar::<Clock>();
    clock.unix_timestamp -= 1;
    f.svm.set_sysvar(&clock);
    let ix = f.rate_instruction(f.authority.pubkey(), SCALE);
    rejected(send(&mut f.svm, &f.authority, ix), "ClockWentBackwards");
    let after: stand_in_yield::YieldVault = read(&f.svm, f.vault);
    assert_eq!(
        (after.price, after.price_time_index, after.updated_at),
        (before.price, before.price_time_index, before.updated_at)
    );
}

#[test]
fn substituted_token_program_cannot_receive_a_cpi() {
    let mut f = YieldFixture::new();
    let mut ix = f.transact(
        f.client.pubkey(),
        stand_in_yield::instruction::Deposit {
            assets: USDC,
            min_shares: USDC,
        }
        .data(),
    );
    ix.accounts
        .iter_mut()
        .find(|a| a.pubkey == spl_token_interface::ID)
        .unwrap()
        .pubkey = solana_system_interface::program::ID;
    rejected(send(&mut f.svm, &f.client, ix), "InvalidProgramId");
    assert_eq!(balance(&f.svm, f.assets), 1040 * USDC);
    assert_eq!(balance(&f.svm, f.reserve), 0);
}

fn load_fee_program(f: &mut YieldFixture) -> Pubkey {
    load(&mut f.svm, advisor_fee_vault::ID, "advisor_fee_vault");
    let loader = anchor_lang::solana_program::bpf_loader_upgradeable::ID;
    let program_data = Pubkey::find_program_address(&[advisor_fee_vault::ID.as_ref()], &loader).0;
    let mut account = f.svm.get_account(&program_data).unwrap();
    let metadata = UpgradeableLoaderState::ProgramData {
        slot: 0,
        upgrade_authority_address: Some(f.authority.pubkey()),
    };
    let len = UpgradeableLoaderState::size_of_programdata_metadata();
    UpgradeableLoaderState::serialize_into(&mut account.data[..len], &metadata).unwrap();
    f.svm.set_account(program_data, account).unwrap();
    program_data
}

fn config_instruction(
    f: &YieldFixture,
    authority: Pubkey,
    program_data: Pubkey,
) -> (Pubkey, Instruction) {
    let config = Pubkey::find_program_address(&[b"config"], &advisor_fee_vault::ID).0;
    (
        config,
        Instruction {
            program_id: advisor_fee_vault::ID,
            accounts: advisor_fee_vault::accounts::InitializeConfig {
                authority,
                program: advisor_fee_vault::ID,
                program_data,
                mint: f.mint,
                config,
                system_program: solana_system_interface::program::ID,
            }
            .to_account_metas(None),
            data: advisor_fee_vault::instruction::InitializeConfig {
                treasury: f.authority.pubkey(),
                demo: false,
            }
            .data(),
        },
    )
}

#[test]
fn config_bootstrap_requires_the_real_upgrade_authority() {
    let mut f = YieldFixture::new();
    let program_data = load_fee_program(&mut f);
    let (config, ix) = config_instruction(&f, f.client.pubkey(), program_data);
    rejected(send(&mut f.svm, &f.client, ix), "Unauthorized");
    assert!(f.svm.get_account(&config).is_none());
    let (_, ix) = config_instruction(&f, f.authority.pubkey(), program_data);
    send(&mut f.svm, &f.authority, ix.clone()).unwrap();
    let state: advisor_fee_vault::Config = read(&f.svm, config);
    assert_eq!(state.authority, f.authority.pubkey());
    assert_eq!(state.settlement_interval, advisor_fee_vault::MONTH_SECONDS);
    assert!(send(&mut f.svm, &f.authority, ix).is_err());
}

#[test]
fn office_and_advisor_enforce_rates_and_roles_onchain() {
    let mut f = YieldFixture::new();
    let program_data = load_fee_program(&mut f);
    let (config, ix) = config_instruction(&f, f.authority.pubkey(), program_data);
    send(&mut f.svm, &f.authority, ix).unwrap();
    let office_owner = Keypair::new();
    f.svm
        .airdrop(&office_owner.pubkey(), 2_000_000_000)
        .unwrap();
    let office = Pubkey::find_program_address(
        &[b"office", config.as_ref(), office_owner.pubkey().as_ref()],
        &advisor_fee_vault::ID,
    )
    .0;
    let register_office = |fee_bps| Instruction {
        program_id: advisor_fee_vault::ID,
        accounts: advisor_fee_vault::accounts::RegisterOffice {
            authority: f.authority.pubkey(),
            config,
            office_authority: office_owner.pubkey(),
            office,
            system_program: solana_system_interface::program::ID,
        }
        .to_account_metas(None),
        data: advisor_fee_vault::instruction::RegisterOffice {
            treasury: office_owner.pubkey(),
            fee_bps,
            advisor_allotment_bps: 60,
        }
        .data(),
    };
    let too_high = register_office(41);
    let valid = register_office(40);
    rejected(send(&mut f.svm, &f.authority, too_high), "FeeCeiling");
    assert!(f.svm.get_account(&office).is_none());
    send(&mut f.svm, &f.authority, valid).unwrap();
    let advisor = Pubkey::find_program_address(
        &[b"advisor", office.as_ref(), f.client.pubkey().as_ref()],
        &advisor_fee_vault::ID,
    )
    .0;
    let ix = Instruction {
        program_id: advisor_fee_vault::ID,
        accounts: advisor_fee_vault::accounts::RegisterAdvisor {
            authority: office_owner.pubkey(),
            office,
            advisor_authority: f.client.pubkey(),
            advisor,
            system_program: solana_system_interface::program::ID,
        }
        .to_account_metas(None),
        data: advisor_fee_vault::instruction::RegisterAdvisor {
            treasury: f.client.pubkey(),
            fee_bps: 60,
        }
        .data(),
    };
    send(&mut f.svm, &office_owner, ix).unwrap();
    let set_advisor = |authority, fee_bps| Instruction {
        program_id: advisor_fee_vault::ID,
        accounts: advisor_fee_vault::accounts::SetAdvisorFee {
            authority,
            office,
            advisor,
        }
        .to_account_metas(None),
        data: advisor_fee_vault::instruction::SetAdvisorFee { fee_bps }.data(),
    };
    let invalid = set_advisor(f.client.pubkey(), 61);
    let wrong_signer = set_advisor(office_owner.pubkey(), 50);
    let valid = set_advisor(f.client.pubkey(), 50);
    rejected(send(&mut f.svm, &f.client, invalid), "AdvisorAllotment");
    rejected(
        send(&mut f.svm, &office_owner, wrong_signer),
        "ConstraintSeeds",
    );
    assert_eq!(
        read::<advisor_fee_vault::Advisor>(&f.svm, advisor).fee_bps,
        60
    );
    send(&mut f.svm, &f.client, valid).unwrap();
    assert_eq!(
        read::<advisor_fee_vault::Advisor>(&f.svm, advisor).fee_bps,
        50
    );
}

use advisor_fee_vault::positions::{Position, Vault};

fn fee_ix(accounts: impl ToAccountMetas, data: impl InstructionData) -> Instruction {
    Instruction {
        program_id: advisor_fee_vault::ID,
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    }
}

struct FeeFixture {
    y: YieldFixture,
    config: Pubkey,
    vault: Pubkey,
    cash: Pubkey,
    custody: Pubkey,
    office: Pubkey,
    advisor: Pubkey,
    position: Pubkey,
    recipients: [Pubkey; 3],
    treasuries: [Pubkey; 3],
}

impl FeeFixture {
    fn new() -> Self {
        let mut y = YieldFixture::new();
        let program_data = load_fee_program(&mut y);
        let (config, ix) = config_instruction(&y, y.authority.pubkey(), program_data);
        send(&mut y.svm, &y.authority, ix).unwrap();
        let vault = Pubkey::find_program_address(
            &[b"vault", config.as_ref(), y.vault.as_ref()],
            &advisor_fee_vault::ID,
        )
        .0;
        let cash =
            Pubkey::find_program_address(&[b"cash", vault.as_ref()], &advisor_fee_vault::ID).0;
        let custody =
            Pubkey::find_program_address(&[b"custody", vault.as_ref()], &advisor_fee_vault::ID).0;
        let ix = fee_ix(
            advisor_fee_vault::accounts::RegisterVault {
                authority: y.authority.pubkey(),
                config,
                mint: y.mint,
                source: y.vault,
                shares_mint: y.shares_mint,
                vault,
                cash,
                custody,
                token_program: spl_token_interface::ID,
                system_program: solana_system_interface::program::ID,
            },
            advisor_fee_vault::instruction::RegisterVault {},
        );
        send(&mut y.svm, &y.authority, ix).unwrap();
        let recipients = [
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            y.authority.pubkey(),
        ];
        let treasuries = std::array::from_fn(|_| Pubkey::new_unique());
        for i in 0..3 {
            token_account(&mut y.svm, treasuries[i], y.mint, recipients[i], 0);
        }
        let office = Pubkey::find_program_address(
            &[b"office", config.as_ref(), y.authority.pubkey().as_ref()],
            &advisor_fee_vault::ID,
        )
        .0;
        let ix = fee_ix(
            advisor_fee_vault::accounts::RegisterOffice {
                authority: y.authority.pubkey(),
                config,
                office_authority: y.authority.pubkey(),
                office,
                system_program: solana_system_interface::program::ID,
            },
            advisor_fee_vault::instruction::RegisterOffice {
                treasury: recipients[0],
                fee_bps: 40,
                advisor_allotment_bps: 60,
            },
        );
        send(&mut y.svm, &y.authority, ix).unwrap();
        let advisor = Pubkey::find_program_address(
            &[b"advisor", office.as_ref(), y.client.pubkey().as_ref()],
            &advisor_fee_vault::ID,
        )
        .0;
        let ix = fee_ix(
            advisor_fee_vault::accounts::RegisterAdvisor {
                authority: y.authority.pubkey(),
                office,
                advisor_authority: y.client.pubkey(),
                advisor,
                system_program: solana_system_interface::program::ID,
            },
            advisor_fee_vault::instruction::RegisterAdvisor {
                treasury: recipients[1],
                fee_bps: 60,
            },
        );
        send(&mut y.svm, &y.authority, ix).unwrap();
        let position = Pubkey::find_program_address(
            &[b"position", vault.as_ref(), y.client.pubkey().as_ref()],
            &advisor_fee_vault::ID,
        )
        .0;
        let mut f = Self {
            y,
            config,
            vault,
            cash,
            custody,
            office,
            advisor,
            position,
            recipients,
            treasuries,
        };
        let ix = f.open(f.y.client.pubkey(), [40, 60]);
        send(&mut f.y.svm, &f.y.client, ix).unwrap();
        f
    }

    fn open(&self, client: Pubkey, rates: [u16; 2]) -> Instruction {
        fee_ix(
            advisor_fee_vault::accounts::OpenPosition {
                client,
                config: self.config,
                vault: self.vault,
                source: self.y.vault,
                office: self.office,
                advisor: self.advisor,
                position: Pubkey::find_program_address(
                    &[b"position", self.vault.as_ref(), client.as_ref()],
                    &advisor_fee_vault::ID,
                )
                .0,
                system_program: solana_system_interface::program::ID,
            },
            advisor_fee_vault::instruction::OpenPosition {
                rates,
                recipients: self.recipients,
            },
        )
    }

    fn settlement(&self) -> advisor_fee_vault::accounts::Settlement {
        advisor_fee_vault::accounts::Settlement {
            config: self.config,
            vault: self.vault,
            position: self.position,
            source: self.y.vault,
            mint: self.y.mint,
            shares_mint: self.y.shares_mint,
            reserve: self.y.reserve,
            cash: self.cash,
            custody: self.custody,
            office_assets: self.treasuries[0],
            advisor_assets: self.treasuries[1],
            rivool_assets: self.treasuries[2],
            source_program: stand_in_yield::ID,
            token_program: spl_token_interface::ID,
        }
    }

    fn client_ix(&self, data: impl InstructionData) -> Instruction {
        fee_ix(
            advisor_fee_vault::accounts::ClientTransact {
                client: self.y.client.pubkey(),
                client_assets: self.y.assets,
                settlement: self.settlement(),
            },
            data,
        )
    }

    fn deposit(&mut self, assets: u64) {
        let ix = self.client_ix(advisor_fee_vault::instruction::Deposit {
            assets,
            min_shares: 1,
        });
        send(&mut self.y.svm, &self.y.client, ix).unwrap();
    }

    fn warp(&mut self, seconds: i64) {
        let mut clock: Clock = self.y.svm.get_sysvar();
        clock.unix_timestamp += seconds;
        self.y.svm.set_sysvar(&clock);
    }

    #[allow(clippy::result_large_err)] // Preserve native VM failure logs for rejection assertions.
    fn settle(&mut self) -> TransactionResult {
        let ix = fee_ix(self.settlement(), advisor_fee_vault::instruction::Settle {});
        send(&mut self.y.svm, &self.y.authority, ix)
    }

    fn state(&self) -> Position {
        read(&self.y.svm, self.position)
    }
    fn paid(&self) -> [u64; 3] {
        self.treasuries.map(|key| balance(&self.y.svm, key))
    }
    fn price(&mut self, price: u128) {
        let ix = self.y.rate_instruction(self.y.authority.pubkey(), price);
        send(&mut self.y.svm, &self.y.authority, ix).unwrap();
    }
    fn accrue(&mut self) {
        let ix = fee_ix(
            advisor_fee_vault::accounts::Accrue {
                vault: self.vault,
                source: self.y.vault,
                position: self.position,
            },
            advisor_fee_vault::instruction::Accrue {},
        );
        send(&mut self.y.svm, &self.y.authority, ix).unwrap();
    }
    fn exit(&mut self) {
        let ix = self.client_ix(advisor_fee_vault::instruction::Withdraw {
            shares: None,
            min_assets: 0,
        });
        send(&mut self.y.svm, &self.y.client, ix).unwrap();
    }
}

#[test]
fn fee_vault_thirty_day_demo_conserves_tokens_and_closes() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    f.warp(30 * 86400);
    f.y.fund_yield();
    f.price(SCALE * 104 / 100);
    f.settle().unwrap();
    assert_eq!(f.paid(), [328_767, 493_150, 6 * USDC]);
    let p = f.state();
    assert_eq!(p.high_water_mark, SCALE * 104 / 100);
    assert_eq!(balance(&f.y.svm, f.cash), p.cash);
    assert_eq!(balance(&f.y.svm, f.custody), p.shares);
    rejected(f.settle(), "NotDue");
    f.exit();
    assert_eq!(f.state().shares, 0);
    assert_eq!(f.state().cash, 0);
    let held = balance(&f.y.svm, f.y.assets)
        + balance(&f.y.svm, f.y.reserve)
        + balance(&f.y.svm, f.cash)
        + f.paid().iter().sum::<u64>();
    assert_eq!(held, 1040 * USDC);
    assert!(balance(&f.y.svm, f.y.reserve) <= 1);
    let ix = fee_ix(
        advisor_fee_vault::accounts::ClosePosition {
            client: f.y.client.pubkey(),
            position: f.position,
        },
        advisor_fee_vault::instruction::ClosePosition {},
    );
    send(&mut f.y.svm, &f.y.client, ix).unwrap();
    assert!(f
        .y
        .svm
        .get_account(&f.position)
        .is_none_or(|a| a.lamports == 0));
    let ix = f.open(f.y.client.pubkey(), [40, 60]);
    send(&mut f.y.svm, &f.y.client, ix).unwrap();
    assert_eq!(f.state().shares, 0);
}

#[test]
fn six_month_operations_conserve_tokens_and_preserve_loss_high_water_mark() {
    let mut f = FeeFixture::new();
    // Fixture capital includes 1,000 invested, 500 for a top-up and 200 for yield backing.
    mint_account(
        &mut f.y.svm,
        f.y.mint,
        f.y.authority.pubkey(),
        1700 * USDC,
        6,
    );
    token_account(
        &mut f.y.svm,
        f.y.assets,
        f.y.mint,
        f.y.client.pubkey(),
        1700 * USDC,
    );
    f.deposit(1000 * USDC);
    let mut previous_price = SCALE;
    for (month, percent) in [104, 98, 100, 104, 108, 110].into_iter().enumerate() {
        let before = f.state();
        let paid_before = f.paid();
        f.warp(30 * 86400);
        if matches!(month, 0 | 3 | 4 | 5) {
            f.y.fund_yield();
        }
        let price = SCALE * percent / 100;
        f.price(price);
        f.settle().unwrap();
        let fees: [u64; 3] = std::array::from_fn(|i| f.paid()[i] - paid_before[i]);
        // Independent integer oracle includes cash and fractional carry across all six months.
        let denominator = SCALE * 10_000 * 365 * 86400;
        for (i, rate) in [40_u128, 60].into_iter().enumerate() {
            let numerator = (u128::from(before.shares) * previous_price
                + u128::from(before.cash) * SCALE)
                * 30
                * 86400
                * rate
                + before.remainders[i];
            assert_eq!(u128::from(fees[i]), numerator / denominator);
            assert_eq!(f.state().remainders[i], numerator % denominator);
        }
        let performance =
            u128::from(before.shares) * price.saturating_sub(before.high_water_mark) * 1500
                + before.remainders[2];
        assert_eq!(u128::from(fees[2]), performance / (SCALE * 10_000));
        assert_eq!(f.state().high_water_mark, before.high_water_mark.max(price));
        if (1..=2).contains(&month) {
            assert_eq!(fees[2], 0, "No performance fee below the blended mark");
        }
        rejected(f.settle(), "NotDue");
        let action = match month {
            1 => {
                f.deposit(500 * USDC);
                assert_eq!(f.paid(), std::array::from_fn(|i| paid_before[i] + fees[i]));
                assert!(f.state().high_water_mark < before.high_water_mark);
                "top-up 500"
            }
            3 => {
                let shares = f.state().shares / 4;
                let mark = f.state().high_water_mark;
                let ix = f.client_ix(advisor_fee_vault::instruction::Withdraw {
                    shares: Some(shares),
                    min_assets: shares,
                });
                send(&mut f.y.svm, &f.y.client, ix).unwrap();
                assert_eq!(f.state().high_water_mark, mark);
                "withdraw 25% of shares"
            }
            5 => {
                f.exit();
                "full exit"
            }
            _ => "settle",
        };
        let p = f.state();
        assert_eq!(balance(&f.y.svm, f.cash), p.cash);
        assert_eq!(balance(&f.y.svm, f.custody), p.shares);
        let held = balance(&f.y.svm, f.y.assets)
            + balance(&f.y.svm, f.y.reserve)
            + balance(&f.y.svm, f.cash)
            + f.paid().iter().sum::<u64>();
        assert_eq!(held, 1700 * USDC);
        println!("month={} price={} fees={fees:?} action={action} shares={} cash={} hwm={} client={} reserve={} conserved={held}",
            month + 1, price, p.shares, p.cash, p.high_water_mark,
            balance(&f.y.svm, f.y.assets), balance(&f.y.svm, f.y.reserve));
        previous_price = price;
    }
    assert_eq!(f.state().shares, 0);
    assert_eq!(f.state().cash, 0);
    let shares = Mint::unpack(&f.y.svm.get_account(&f.y.shares_mint).unwrap().data).unwrap();
    assert_eq!(shares.supply, 0);
    println!("six-month final recipient atoms={:?}", f.paid());
}

#[test]
fn accrual_frequency_and_price_changes_do_not_change_fees() {
    fn run(frequent: bool) -> ([u64; 3], [u128; 3]) {
        let mut f = FeeFixture::new();
        f.deposit(1000 * USDC);
        for day in 0..30 {
            f.warp(86400);
            if day == 9 {
                f.price(SCALE / 2);
            }
            if day == 19 {
                f.price(SCALE);
            }
            if frequent {
                f.accrue();
            }
        }
        f.settle().unwrap();
        assert_eq!(f.paid()[2], 0);
        (f.paid(), f.state().remainders)
    }
    assert_eq!(run(true), run(false));
}

#[test]
fn accrue_does_not_crystallize_transient_profit_and_losses_keep_the_mark() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    f.y.fund_yield();
    f.price(SCALE * 104 / 100);
    f.accrue();
    assert_eq!(f.state().high_water_mark, SCALE);
    f.price(SCALE / 2);
    f.warp(30 * 86400);
    f.settle().unwrap();
    assert_eq!(f.paid()[2], 0);
    assert_eq!(f.state().high_water_mark, SCALE);
    f.price(SCALE);
    f.exit();
    assert_eq!(f.paid()[2], 0);
}

#[test]
fn accepted_rates_survive_profile_changes_and_stale_terms_are_rejected() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    let ix = fee_ix(
        advisor_fee_vault::accounts::SetAdvisorFee {
            authority: f.y.client.pubkey(),
            office: f.office,
            advisor: f.advisor,
        },
        advisor_fee_vault::instruction::SetAdvisorFee { fee_bps: 10 },
    );
    send(&mut f.y.svm, &f.y.client, ix).unwrap();
    let other = Keypair::new();
    f.y.svm.airdrop(&other.pubkey(), 100_000_000).unwrap();
    let ix = f.open(other.pubkey(), [40, 60]);
    rejected(send(&mut f.y.svm, &other, ix), "TermsChanged");
    let ix = f.open(other.pubkey(), [40, 10]);
    send(&mut f.y.svm, &other, ix).unwrap();
    f.warp(30 * 86400);
    f.settle().unwrap();
    assert_eq!(f.paid(), [328_767, 493_150, 0]);
}

#[test]
fn topup_settles_whole_position_and_blends_loss_basis() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    f.warp(30 * 86400);
    f.price(SCALE / 2);
    f.deposit(40 * USDC);
    assert_eq!(f.paid(), [328_767, 493_150, 0]);
    let p = f.state();
    assert_eq!(p.owed, [0, 0]);
    assert!(p.high_water_mark > SCALE / 2 && p.high_water_mark < SCALE);
    let expected = advisor_fee_vault::math::blended_high_water_mark(
        1000 * USDC - 821_917 * 2,
        SCALE,
        80 * USDC,
        SCALE / 2,
    )
    .unwrap();
    assert_eq!(p.high_water_mark, expected);
    let ix = f.client_ix(advisor_fee_vault::instruction::Withdraw {
        shares: Some(80 * USDC),
        min_assets: 40 * USDC,
    });
    send(&mut f.y.svm, &f.y.client, ix).unwrap();
    assert_eq!(f.state().high_water_mark, expected);
    assert_eq!(balance(&f.y.svm, f.y.assets), 40 * USDC);
}

#[test]
fn authorization_substitution_and_slippage_roll_back_all_fee_payments() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    f.warp(30 * 86400);
    let before = f.y.svm.get_account(&f.position).unwrap().data;
    let ix = f.client_ix(advisor_fee_vault::instruction::Withdraw {
        shares: None,
        min_assets: 1000 * USDC,
    });
    rejected(send(&mut f.y.svm, &f.y.client, ix), "Slippage");
    assert_eq!(f.paid(), [0; 3]);
    assert_eq!(f.y.svm.get_account(&f.position).unwrap().data, before);
    let ix = fee_ix(
        advisor_fee_vault::accounts::ClientTransact {
            client: f.y.authority.pubkey(),
            client_assets: f.treasuries[2],
            settlement: f.settlement(),
        },
        advisor_fee_vault::instruction::Withdraw {
            shares: None,
            min_assets: 0,
        },
    );
    rejected(send(&mut f.y.svm, &f.y.authority, ix), "Unauthorized");
    let mut accounts = f.settlement();
    accounts.rivool_assets = f.y.assets;
    let ix = fee_ix(accounts, advisor_fee_vault::instruction::Settle {});
    rejected(
        send(&mut f.y.svm, &f.y.authority, ix),
        "ConstraintTokenOwner",
    );
    let ix = fee_ix(
        advisor_fee_vault::accounts::ClosePosition {
            client: f.y.client.pubkey(),
            position: f.position,
        },
        advisor_fee_vault::instruction::ClosePosition {},
    );
    rejected(send(&mut f.y.svm, &f.y.client, ix), "PositionFunded");
    assert_eq!(f.paid(), [0; 3]);
    assert_eq!(balance(&f.y.svm, f.custody), 1000 * USDC);
}

#[test]
fn exhausted_fees_cannot_charge_new_deposits_and_dust_can_exit() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    f.warp(101 * 365 * 86400);
    f.settle().unwrap();
    assert_eq!(f.paid(), [400 * USDC, 600 * USDC, 0]);
    assert_eq!(f.state().shares, 0);
    assert_eq!(f.state().remainders, [0; 3]);
    f.deposit(40 * USDC);
    f.exit();
    assert_eq!(balance(&f.y.svm, f.y.assets), 40 * USDC);
    f.deposit(1);
    f.price(1);
    f.exit();
    assert_eq!(f.state().shares, 0);
}

#[test]
fn two_clients_keep_independent_assets_rates_and_rounding() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    let first_position = f.position;
    let ix = fee_ix(
        advisor_fee_vault::accounts::SetAdvisorFee {
            authority: f.y.client.pubkey(),
            office: f.office,
            advisor: f.advisor,
        },
        advisor_fee_vault::instruction::SetAdvisorFee { fee_bps: 10 },
    );
    send(&mut f.y.svm, &f.y.client, ix).unwrap();
    let other = Keypair::new();
    f.y.svm.airdrop(&other.pubkey(), 100_000_000).unwrap();
    let ix = f.open(other.pubkey(), [40, 10]);
    send(&mut f.y.svm, &other, ix).unwrap();
    f.position = Pubkey::find_program_address(
        &[b"position", f.vault.as_ref(), other.pubkey().as_ref()],
        &advisor_fee_vault::ID,
    )
    .0;
    let other_assets = Pubkey::new_unique();
    // Transfer the remaining 40 tokens to the second client, conserving the fixture supply.
    token_account(&mut f.y.svm, f.y.assets, f.y.mint, f.y.client.pubkey(), 0);
    token_account(
        &mut f.y.svm,
        other_assets,
        f.y.mint,
        other.pubkey(),
        40 * USDC,
    );
    let ix = fee_ix(
        advisor_fee_vault::accounts::ClientTransact {
            client: other.pubkey(),
            client_assets: other_assets,
            settlement: f.settlement(),
        },
        advisor_fee_vault::instruction::Deposit {
            assets: 40 * USDC,
            min_shares: 40 * USDC,
        },
    );
    send(&mut f.y.svm, &other, ix).unwrap();
    f.warp(30 * 86400);
    let untouched = f.y.svm.get_account(&first_position).unwrap().data;
    f.settle().unwrap();
    assert_eq!(f.paid(), [13_150, 3287, 0]);
    assert_eq!(
        f.y.svm.get_account(&first_position).unwrap().data,
        untouched
    );
    let second_shares = f.state().shares;
    f.position = first_position;
    f.settle().unwrap();
    assert_eq!(
        read::<Vault>(&f.y.svm, f.vault).total_shares,
        f.state().shares + second_shares
    );
    assert_eq!(
        balance(&f.y.svm, f.custody),
        f.state().shares + second_shares
    );
    assert_eq!(f.paid(), [328_767 + 13_150, 493_150 + 3287, 0]);
}

#[test]
fn authority_rotation_preserves_positions_and_revokes_old_signer() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    let next = Keypair::new();
    f.y.svm.airdrop(&next.pubkey(), 100_000_000).unwrap();
    let ix = fee_ix(
        advisor_fee_vault::accounts::SetOfficeFee {
            authority: f.y.authority.pubkey(),
            office: f.office,
        },
        advisor_fee_vault::instruction::TransferOfficeAuthority {
            authority: next.pubkey(),
        },
    );
    send(&mut f.y.svm, &f.y.authority, ix).unwrap();
    let set_fee = |authority| {
        fee_ix(
            advisor_fee_vault::accounts::SetOfficeFee {
                authority,
                office: f.office,
            },
            advisor_fee_vault::instruction::SetOfficeFee {
                fee_bps: 20,
                advisor_allotment_bps: 60,
            },
        )
    };
    let ix = set_fee(f.y.authority.pubkey());
    rejected(send(&mut f.y.svm, &f.y.authority, ix), "ConstraintHasOne");
    let ix = set_fee(next.pubkey());
    send(&mut f.y.svm, &next, ix).unwrap();
    assert_eq!(f.state().rates, [40, 60]);
    let ix = fee_ix(
        advisor_fee_vault::accounts::TransferConfigAuthority {
            authority: f.y.authority.pubkey(),
            config: f.config,
        },
        advisor_fee_vault::instruction::TransferConfigAuthority {
            authority: next.pubkey(),
        },
    );
    send(&mut f.y.svm, &f.y.authority, ix.clone()).unwrap();
    rejected(send(&mut f.y.svm, &f.y.authority, ix), "ConstraintHasOne");
    assert_eq!(
        read::<advisor_fee_vault::Config>(&f.y.svm, f.config).authority,
        next.pubkey()
    );
    f.warp(30 * 86400);
    f.settle().unwrap();
    assert_eq!(f.paid(), [328_767, 493_150, 0]);
    f.exit();
}

#[test]
fn partial_exit_and_failed_topup_settle_all_earned_fees_atomically() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    f.warp(30 * 86400);
    let before = f.y.svm.get_account(&f.position).unwrap().data;
    let ix = f.client_ix(advisor_fee_vault::instruction::Deposit {
        assets: USDC,
        min_shares: u64::MAX,
    });
    rejected(send(&mut f.y.svm, &f.y.client, ix), "Slippage");
    assert_eq!(f.y.svm.get_account(&f.position).unwrap().data, before);
    assert_eq!(f.paid(), [0; 3]);
    assert_eq!(balance(&f.y.svm, f.y.assets), 40 * USDC);
    let ix = f.client_ix(advisor_fee_vault::instruction::Withdraw {
        shares: Some(100 * USDC),
        min_assets: 100 * USDC,
    });
    send(&mut f.y.svm, &f.y.client, ix).unwrap();
    assert_eq!(f.paid(), [328_767, 493_150, 0]);
    assert_eq!(f.state().shares, 900 * USDC - 821_917);
    assert_eq!(balance(&f.y.svm, f.y.assets), 140 * USDC);
}

#[test]
fn redemption_rounding_cash_is_position_owned_and_accrues_exactly() {
    let mut f = FeeFixture::new();
    f.deposit(USDC);
    f.warp(30 * 86400);
    f.y.fund_yield();
    f.price(SCALE * 3);
    f.settle().unwrap();
    assert_eq!(f.paid(), [328, 493, 300_000]);
    let p = f.state();
    assert_eq!(p.cash, 1);
    assert_eq!(balance(&f.y.svm, f.cash), 1);
    f.warp(30 * 86400);
    f.accrue();
    for i in 0..2 {
        let share_fee = advisor_fee_vault::math::advisory_fee(
            p.shares,
            SCALE * 3 * 30 * 86400,
            p.rates[i],
            p.remainders[i],
        )
        .unwrap();
        let cash_fee = advisor_fee_vault::math::advisory_fee(
            1,
            SCALE * 30 * 86400,
            p.rates[i],
            share_fee.remainder,
        )
        .unwrap();
        assert_eq!(f.state().owed[i], share_fee.amount + cash_fee.amount);
        assert_eq!(f.state().remainders[i], cash_fee.remainder);
    }
    f.exit();
    assert_eq!(balance(&f.y.svm, f.cash), 0);
    assert_eq!(f.state().cash, 0);
    assert_eq!(
        balance(&f.y.svm, f.y.assets)
            + balance(&f.y.svm, f.y.reserve)
            + f.paid().iter().sum::<u64>(),
        1040 * USDC
    );
}

#[test]
fn failure_of_third_recipient_rolls_back_both_earlier_payments_and_redemption() {
    let mut f = FeeFixture::new();
    f.deposit(1000 * USDC);
    f.warp(30 * 86400);
    f.y.fund_yield();
    f.price(SCALE * 104 / 100);
    let mut account = f.y.svm.get_account(&f.treasuries[2]).unwrap();
    let mut token = TokenAccount::unpack(&account.data).unwrap();
    token.state = AccountState::Frozen;
    TokenAccount::pack(token, &mut account.data).unwrap();
    f.y.svm.set_account(f.treasuries[2], account).unwrap();
    let before = f.y.svm.get_account(&f.position).unwrap().data;
    rejected(f.settle(), "Account is frozen");
    assert_eq!(f.paid(), [0; 3]);
    assert_eq!(balance(&f.y.svm, f.y.reserve), 1040 * USDC);
    assert_eq!(balance(&f.y.svm, f.custody), 1000 * USDC);
    assert_eq!(f.y.svm.get_account(&f.position).unwrap().data, before);
}

#[test]
fn token_2022_mint_is_rejected_at_bootstrap() {
    let mut f = YieldFixture::new();
    let program_data = load_fee_program(&mut f);
    let mut mint = f.svm.get_account(&f.mint).unwrap();
    mint.owner = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        .parse()
        .unwrap();
    f.svm.set_account(f.mint, mint).unwrap();
    let (config, ix) = config_instruction(&f, f.authority.pubkey(), program_data);
    rejected(
        send(&mut f.svm, &f.authority, ix),
        "AccountOwnedByWrongProgram",
    );
    assert!(f.svm.get_account(&config).is_none());
}
