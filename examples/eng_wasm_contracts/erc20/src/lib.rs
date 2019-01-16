#![no_std]
#![feature(proc_macro_gen)]
#![feature(proc_macro_non_items)]


extern crate eng_wasm;
extern crate eng_wasm_derive;
extern crate rustc_hex as hex;
#[macro_use]
extern crate more_asserts;
#[macro_use]
extern crate serde_derive;
extern crate serde;

use eng_wasm::*;
use eng_wasm_derive::pub_interface;
use eng_wasm::String;
use eng_wasm::from_utf8;
use hex::ToHex;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

static TOTAL_SUPPLY: &str = "total_supply";

#[pub_interface]
pub trait Erc20Interface{
    /// 'get_user' receives an address and returns it's User object,
    /// if it does not exist, it creates a new object.
    fn get_user(user: H256) -> User;
    /// 'call_ret' sends a U256 result to the external ret function.
    fn call_ret(res: U256);
    /// 'mint' creates new tokens and sends to the specified address
    fn mint(addr: H256, tokens: U256);
    /// receive the total_supply
    fn total_supply() -> U256;
    /// receive the balance of the specified address
    fn balance_of(token_owner: H256) -> U256;
    /// receive the allowed amount of the owner tokens to be spent by the spender address
    fn allowance(owner: H256, spender: H256) -> U256;
    /// transfer tokens from 'from' address to the 'to' address
    fn transfer(from: H256, to: H256, tokens: U256);
    /// approve the 'spender' address to spend 'tokens' from the 'owner's address balance
    fn approve(token_owner: H256, spender: H256, tokens: U256);
    /// 'spender' address transfers tokens on behalf of the owner address to the 'to' address
    fn transfer_from(owner: H256, spender: H256, to: H256, tokens: U256);
}

/// User object holds all information of a user address
#[derive(Serialize, Deserialize)]
pub struct User {
    /// 'balance' is the number of tokens a user owns
    balance : u64,
    /// 'approved' is a HashMap which contains all authorised spenders by the user
    approved : HashMap<String, u64>,
}

pub struct Contract;
impl Erc20Interface for Contract {

    #[no_mangle]
    fn get_user(user: H256) -> User {
        match read_state!(&user.to_hex()) {
            Some(user) => user,
            // if does not exist, create a new user object
            None => User{balance: 0, approved: HashMap::new()},
        }
    }

    #[no_mangle]
    fn call_ret(res: U256) {
        let mut byte_res = [0u8; 32];
        res.to_big_endian(&mut byte_res);
        unsafe {external::ret(byte_res.as_ptr(), byte_res.len() as u32)};
    }

    #[no_mangle]
    fn mint(addr: H256, tokens: U256) {
        let total_supply: u64 = match read_state!(TOTAL_SUPPLY) {
            Some(amount) => amount,
            None => 0,
        };
        let mut user_addr = Self::get_user(addr);

        // update the user object and write to the state
        user_addr.balance = user_addr.balance + tokens.as_u64();
        write_state!(&addr.to_hex() => user_addr, TOTAL_SUPPLY => (total_supply + tokens.as_u64()));
    }

    #[no_mangle]
    fn total_supply() -> U256 {
        let total: U256 = match read_state!(TOTAL_SUPPLY) {
            Some(amount) => amount,
            None => 0,
        }.into();
        // todo: remove the call to ret once it is changed on eng_wasm
        Self::call_ret(total);
        total
    }

    #[no_mangle]
    fn balance_of(token_owner: H256) -> U256 {
        let user: User = Self::get_user(token_owner);
        Self::call_ret(user.balance.into());
        user.balance.into()
    }

    #[no_mangle]
    fn allowance(owner: H256, spender: H256) -> U256 {
        let user: User = Self::get_user(owner);
        let approved_bal: U256 = match user.approved.get(&spender.to_hex()) {
            Some(amount) => *amount,
            None => 0,
        }.into();
        Self::call_ret(approved_bal);
        approved_bal
    }

    #[no_mangle]
    fn transfer(from: H256, to: H256, tokens: U256) {
        let mut from_user : User = Self::get_user(from);

        // panic if the 'from' address does not have enough tokens.
        assert_gt!(from_user.balance, tokens.as_u64(), "invalid action: user does not have enough tokens");
        let mut to_user : User = Self::get_user(to);

        // update the balances and write the user objects to the state
        from_user.balance = from_user.balance - tokens.as_u64();
        to_user.balance = to_user.balance + tokens.as_u64();
        write_state!(&from.to_hex() => from_user, &to.to_hex() => to_user);
    }

    #[no_mangle]
    fn approve(token_owner: H256, spender: H256, tokens: U256){
        let mut owner_user : User = Self::get_user(token_owner);
        assert_gt!(owner_user.balance, tokens.as_u64(), "invalid action: owner does not have enough tokens");

        // update the object and write it to the state
        owner_user.approved.insert(spender.to_hex(), tokens.as_u64());
        write_state!(&token_owner.to_hex() => owner_user);
    }

    #[no_mangle]
    fn transfer_from(owner: H256, spender: H256, to: H256, tokens: U256) {
        let mut owner_user : User = Self::get_user(owner);
        // panic if the owner does not own the amount of tokens
        assert_gt!(owner_user.balance, tokens.as_u64(), "invalid action: owner does not have enough tokens");

        let allowed_balance: u64 = match owner_user.approved.get(&spender.to_hex()) {
            Some(amount) => *amount,
            None => 0,
        };
        // panic if the spender is not approved to spend as much as tokens
        assert_gt!(allowed_balance, tokens.as_u64(), "invalid action: user is not allowed to spend this amount of tokens");

        let mut to_user: User = Self::get_user(to);

        // update the objects and write to the state
        to_user.balance = to_user.balance + tokens.as_u64();
        owner_user.balance = owner_user.balance - tokens.as_u64();
        owner_user.approved.insert(spender.to_hex(),(allowed_balance - tokens.as_u64()));
        write_state!(&owner.to_hex() => owner_user, &to.to_hex() => to_user);
    }
}