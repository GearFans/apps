// Fungible Token Smart Contract.
// Implementation based on https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/ERC20.sol

#![no_std]
#![feature(const_btree_new)]

use fungible_token_messages::{Action, ApproveReply, Event, InitConfig, TransferReply};
use gstd::{debug, msg, prelude::*, ActorId};
use primitive_types::H256;

const GAS_AMOUNT: u64 = 300_000_000;

#[derive(Debug)]
struct FungibleToken {
    /// name of the token.
    name: String,
    /// symbol of the token.
    symbol: String,
    /// total supply of the token.
    total_supply: u128,
    /// map to hold balances of token holders.
    balances: BTreeMap<ActorId, u128>,
    /// map to hold allowance information of token holders.
    allowances: BTreeMap<ActorId, BTreeMap<ActorId, u128>>,
    /// creater of the token.
    creator: ActorId,
    /// creater approved set of admins, who can do mint/burn.
    admins: BTreeSet<ActorId>,
}

static mut FUNGIBLE_TOKEN: FungibleToken = FungibleToken {
    name: String::new(),
    symbol: String::new(),
    total_supply: 0,
    balances: BTreeMap::new(),
    allowances: BTreeMap::new(),
    creator: ActorId::new(H256::zero().to_fixed_bytes()),
    admins: BTreeSet::new(),
};

impl FungibleToken {
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn set_symbol(&mut self, symbol: String) {
        self.symbol = symbol;
    }
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn add_admin(&mut self, account: &ActorId) {
        unsafe {
            let source = msg::source();
            if FUNGIBLE_TOKEN.creator != source {
                panic!("fungibletoken: only token creater can add admin.");
            }
            if *account != FUNGIBLE_TOKEN.creator {
                self.admins.insert(*account);
            }
            msg::send(msg::source(), Event::AdminAdded(*account), GAS_AMOUNT, 0);
        }
    }
    fn remove_admin(&mut self, account: &ActorId) {
        unsafe {
            let source = msg::source();
            if FUNGIBLE_TOKEN.creator != source {
                panic!("FungibleToken: Only token creater can remove admin.");
            }
            if *account != FUNGIBLE_TOKEN.creator {
                self.admins.remove(account);
            }
            msg::send(msg::source(), Event::AdminRemoved(*account), GAS_AMOUNT, 0);
        }
    }
    fn total_supply(&self) {
        msg::send(msg::source(), Event::TotalIssuance(self.total_supply), GAS_AMOUNT, 0);
    }
    #[allow(dead_code)]
    fn decimals(&self) -> u8 {
        18
    }
    fn increase_total_supply(&mut self, amount: u128) {
        self.total_supply = self.total_supply.saturating_add(amount);
    }
    fn decrease_total_supply(&mut self, amount: u128) {
        self.total_supply = self.total_supply.saturating_sub(amount);
    }
    fn set_balance(&mut self, account: &ActorId, amount: u128) {
        self.balances.insert(*account, amount);
    }
    fn get_balance(&self, account: &ActorId) -> u128 {
        *self.balances.get(account).unwrap_or(&0)
    }
    fn balance_of(&self, account: &ActorId) {
        let balance = self.get_balance(account);
        msg::send(msg::source(), Event::Balance(balance), GAS_AMOUNT, 0);
    }
    fn mint(&mut self, account: &ActorId, amount: u128) {
        unsafe {
            let source = msg::source();
            if FUNGIBLE_TOKEN.creator != source && !FUNGIBLE_TOKEN.admins.contains(&source) {
                panic!("FungibleToken: Only token creater or designated admins can mint tokens.");
            }
        }
        // debug!("mint to account {:?}", account);
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        if account == &zero {
            panic!("FungibleToken: Mint to zero address.");
        }
        unsafe {
            self.increase_total_supply(amount);
            let old_balance = FUNGIBLE_TOKEN.get_balance(account);
            self.set_balance(account, old_balance.saturating_add(amount));
        }
        let transfer_data = TransferReply {
            from: zero,
            to: *account,
            amount,
        };
        msg::send(msg::source(), Event::Transfer(transfer_data), GAS_AMOUNT, 0);
    }
    fn burn(&mut self, account: &ActorId, amount: u128) {
        unsafe {
            let source = msg::source();
            if FUNGIBLE_TOKEN.creator != source && !FUNGIBLE_TOKEN.admins.contains(&source) {
                panic!("FungibleToken: Only token creater or designated admins can burn tokens.");
            }
        }
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        if account == &zero {
            panic!("FungibleToken: Burn from zero address.");
        }
        unsafe {
            self.decrease_total_supply(amount);
            let old_balance = FUNGIBLE_TOKEN.get_balance(account);
            self.set_balance(account, old_balance.saturating_sub(amount));
        }
        let transfer_data = TransferReply {
            from: *account,
            to: zero,
            amount,
        };
        msg::send(msg::source(), Event::Transfer(transfer_data), GAS_AMOUNT, 0);
    }
    fn transfer(&mut self, sender: &ActorId, recipient: &ActorId, amount: u128) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        if sender == &zero {
            panic!("FungibleToken: Transfer from zero address.");
        }
        if recipient == &zero {
            panic!("FungibleToken: Transfer to zero address.");
        }
        let sender_balance = self.get_balance(sender);
        if amount > sender_balance {
            panic!(
                "FungibleToken: Transfer amount {:?} exceeds sender {:?} balance {:?}.",
                amount, sender, sender_balance
            );
        }
        self.set_balance(sender, sender_balance.saturating_sub(amount));
        let recipient_balance = self.get_balance(recipient);
        self.set_balance(recipient, recipient_balance.saturating_add(amount));
        let transfer_data = TransferReply {
            from: *sender,
            to: *recipient,
            amount,
        };
        msg::send(msg::source(), Event::Transfer(transfer_data), GAS_AMOUNT, 0);
    }
    fn approve(&mut self, owner: &ActorId, spender: &ActorId, amount: u128) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        if spender == &zero {
            panic!("FungibleToken: Approve to zero address.");
        }
        // debug!(
        //     "{:?} approved {:?} to spend upto {:?}",
        //     owner, spender, amount
        // );
        self.allowances
            .entry(*owner)
            .or_default()
            .insert(*spender, amount);
        let approve_data = ApproveReply {
            owner: *owner,
            spender: *spender,
            amount,
        };
        msg::send(msg::source(), Event::Approval(approve_data), GAS_AMOUNT, 0);
    }
    fn get_allowance(&self, owner: &ActorId, spender: &ActorId) -> u128 {
        *self
            .allowances
            .get(owner)
            .and_then(|m| m.get(spender))
            .unwrap_or(&0)
    }
    fn increase_allowance(&mut self, owner: &ActorId, spender: &ActorId, amount: u128) {
        let allowance = self.get_allowance(owner, spender);
        self.approve(owner, spender, allowance.saturating_add(amount));
    }
    fn decrease_allowance(&mut self, owner: &ActorId, spender: &ActorId, amount: u128) {
        let allowance = self.get_allowance(owner, spender);
        if amount > allowance {
            panic!("FungibleToken: Decreased allowance below zero.");
        }
        self.approve(owner, spender, allowance - amount);
    }
    fn transfer_from(
        &mut self,
        owner: &ActorId,
        sender: &ActorId,
        recipient: &ActorId,
        amount: u128,
    ) {
        // debug!(
        //     "{:?} wants to send {:?} to {:?} on behalf of {:?}",
        //     sender, amount, recipient, owner
        // );
        let current_allowance = self.get_allowance(owner, sender);
        if current_allowance < amount {
            panic!("FungibleToken: Transfer amount exceeds allowance");
        }
        self.transfer(owner, recipient, amount);
        self.approve(owner, sender, current_allowance - amount);
    }
}

gstd::metadata! {
    title: "FungibleToken",
    init:
        input: InitConfig,
    handle:
        input: Action,
        output: Event,
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");

    match action {
        Action::Mint(mint_input) => {
            // let to = ActorId::new(mint_input.account.to_fixed_bytes());
            FUNGIBLE_TOKEN.mint(&mint_input.account, mint_input.amount);
        }
        Action::Burn(burn_input) => {
            // let from = ActorId::new(burn_input.account.to_fixed_bytes());
            FUNGIBLE_TOKEN.burn(&burn_input.account, burn_input.amount);
        }
        Action::Transfer(transfer_data) => {
            let from = msg::source();
            // let to = ActorId::new(transfer_data.to.to_fixed_bytes());
            let to = transfer_data.to;
            FUNGIBLE_TOKEN.transfer(&from, &to, transfer_data.amount);
        }
        Action::Approve(approve_data) => {
            let approver = msg::source();
            // let spender = ActorId::new(approve_data.spender.to_fixed_bytes());
            FUNGIBLE_TOKEN.approve(&approver, &approve_data.spender, approve_data.amount);
        }
        Action::TransferFrom(transfer_data) => {
            // let owner = ActorId::new(transfer_data.owner.to_fixed_bytes());
            let from = msg::source();
            // let to = ActorId::new(transfer_data.to.to_fixed_bytes());
            FUNGIBLE_TOKEN.transfer_from(
                &transfer_data.owner,
                &from,
                &transfer_data.to,
                transfer_data.amount,
            );
        }
        Action::IncreaseAllowance(approve_data) => {
            let approver = msg::source();
            // let spender = ActorId::new(approve_data.spender.to_fixed_bytes());
            FUNGIBLE_TOKEN.increase_allowance(
                &approver,
                &approve_data.spender,
                approve_data.amount,
            );
        }
        Action::DecreaseAllowance(approve_data) => {
            let approver = msg::source();
            // let spender = ActorId::new(approve_data.spender.to_fixed_bytes());
            FUNGIBLE_TOKEN.decrease_allowance(&approver, &approve_data.spender, approve_data.amount)
        }
        Action::TotalIssuance => {
            FUNGIBLE_TOKEN.total_supply();
        }
        Action::BalanceOf(account) => {
            // let account = ActorId::new(acc.to_fixed_bytes());
            FUNGIBLE_TOKEN.balance_of(&account);
        }
        Action::AddAdmin(account) => {
            // let account = ActorId::new(acc.to_fixed_bytes());
            FUNGIBLE_TOKEN.add_admin(&account);
        }
        Action::RemoveAdmin(account) => {
            // let account = ActorId::new(acc.to_fixed_bytes());
            FUNGIBLE_TOKEN.remove_admin(&account);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    debug!("FUNGIBLE_TOKEN {:?}", config);
    FUNGIBLE_TOKEN.set_name(config.name);
    FUNGIBLE_TOKEN.set_symbol(config.symbol);
    FUNGIBLE_TOKEN.creator = msg::source();
    debug!(
        "FUNGIBLE_TOKEN {} SYMBOL {} created",
        FUNGIBLE_TOKEN.name(),
        FUNGIBLE_TOKEN.symbol()
    );
}

#[no_mangle]
pub unsafe extern "C" fn handle_reply() {}
