#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
//use concordium_smart_contract_testing::Transfer;
use concordium_std::{collections::*, *};
use core::fmt::Debug;

#[derive(Serialize, SchemaType, Clone)]
pub struct InitialiseParams{
    pub timeout : Duration,
    #[concordium(size_length = 2)]
    pub signers: BTreeSet<AccountAddress>,

    pub min_signers_req: u16
}
/// Your smart contract state.
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "St")]
pub struct State<St>{
    // Arguments initialising the smart contract
    init_params: InitialiseParams,

    requests: StateMap<RequestId, Request, St>
    // Your state
}

#[derive(Clone, Serialize, SchemaType)]
pub struct Request{
    pub amount: Amount,
    pub sender_account: AccountAddress,
    pub accounts_in_aggrement: BTreeSet<AccountAddress>,
    pub receiver: AccountAddress,
    pub expiry: Timestamp
}

type RequestId = u64;
type CustomisedResult<T> = Result<T, ErrorOnReceive>;

#[derive(Serialize, SchemaType)]
pub enum RequestAction {
     SeekTransfer(Amount, AccountAddress, RequestId),
     AcceptTransfer(Amount, AccountAddress, RequestId),
}

/// Your smart contract errors.
#[derive(Debug, PartialEq, Eq, Reject, Serialize, SchemaType)]
pub enum Error {
    /// Failed parsing the parameter.
    #[from(ParseError)]
    ParseParams,
    // Accounts must be minimum of 3
    IncompleteAccounts,
    // Three accounts must be in support at any given time for transfer to be made else an error is thrown
    // min_signers_req must be equal or less than total accounts 
    AccountsLessThanSupportNeeded
}

#[derive(Debug, PartialEq, Eq, Reject, Serialize, SchemaType)]
pub enum ErrorOnReceive{
    #[from(ParseError)]
    ParseParams,
    // Only accounts allowed to send to this contract are permitted
    NotRegisteredAccount,
    // Time elapsed
    TimedOut,
    // Receiving account not eligible or invalid
    InvalidRecipient,
    // Same Request to transfer can't be made twice ie using same request id
    AlreadyExists,
    // Requested amount more than balance
    InsufficientBalance,
    // Only normal accounts can send to this contract
    NotUserAccount,
    //
    Overflow,
}

/// Init function that creates a new smart contract.
#[init(contract = "multi_sig", parameter = "InitialiseParams", payable)]
fn init<St: HasStateApi>(_ctx: &InitContext, _state_builder: &mut StateBuilder<St>, _amount: Amount,) -> Result<State<St>, Error> {
    // Your code
    let parameter: InitialiseParams = match _ctx.parameter_cursor().get(){
        Ok(parameter) => parameter,
        Err(_) => Err(Error::ParseParams).unwrap(),
    };
    ensure!(parameter.min_signers_req <= parameter.signers.len() as u16, Error::AccountsLessThanSupportNeeded);

    ensure!(parameter.signers.len() >= 3, Error::IncompleteAccounts);

    let params = InitialiseParams{
        timeout: parameter.timeout,
        signers: parameter.signers,
        min_signers_req: parameter.min_signers_req
    };

    let state = State {
        init_params: params.clone(),
        requests: _state_builder.new_map(),
    };

    Ok(state)
}

impl From<TransferError> for ErrorOnReceive {
    fn from(re: TransferError) -> Self{
        match re{
            TransferError::AmountTooLarge => Self::InsufficientBalance,
            TransferError::MissingAccount => Self::InvalidRecipient,
        }
    }
}

/// Receive function. The input parameter is the boolean variable `throw_error`.
///  If `throw_error == true`, the receive function will throw a custom error.
///  If `throw_error == false`, the receive function executes successfully.
#[receive(
    contract = "multi_sig",
    name = "deposit",
    error = "Error",
    payable
)]
//#[inline(always)]
fn deposit<St: HasStateApi>(_ctx: &ReceiveContext, _host: &impl HasHost<State<St>, StateApiType = St>, _amount: Amount,) -> Result<(), Error> {
    // Your code
        Ok(())
}

// Function to enable the owner of the request to cancel as long as the time hasn't elapsed
#[receive(
    contract = "multi_sig",
    name = "cancel",
    error = "Error",
    mutable
)]
//#[inline(always)]
fn cancel_request<St: HasStateApi>(_ctx: &ReceiveContext, _host: &mut impl HasHost<State<St>, StateApiType = St>,) -> Result<(), Error> {
    // Your code
    let sender = _ctx.sender();
    let address = match sender{
        Address::Contract(_) => panic!("Must not be a contract address"),
        Address::Account(address) => address,
    };

    let current_time = _ctx.metadata().slot_time();
    let mut found : bool = false;
    let mut id = 0;
    
    for(y, q) in _host.state().requests.iter() {
        if q.expiry > current_time && q.sender_account == address{
            found = true;
            id = *y;
            break;
        }
    }
    if found == true{
        _host.state_mut().requests.remove(&id);
        Ok(())
    } else{
        Err(Error::ParseParams)
    }
        
}


/// Receive function that returns the status of the state.
#[receive(contract = "multi_sig", mutable, name = "receive", payable, error = "ErrorOnReceive", parameter = "RequestAction")]
fn message<St: HasStateApi>(_ctx: &ReceiveContext, host: &mut impl HasHost<State<St>, StateApiType = St>, amount: Amount) -> CustomisedResult<()> {
let sender = _ctx.sender();

// Get info on the interating address if its a contract or user Account
let address_type = match sender{
    Address::Contract(_) => Err(ErrorOnReceive::NotUserAccount),
    Address::Account(address) => Ok(address),
};
let valid_address = address_type.unwrap();

// Validate that the account is part of those authorised to call this contract
ensure!(host.state().init_params.signers.iter().any(|signers| sender.matches_account(signers)), ErrorOnReceive::NotRegisteredAccount);

let now = _ctx.metadata().slot_time();
let ozi = _ctx.parameter_cursor().get().unwrap();
match ozi {
    RequestAction::SeekTransfer(transfer_amount, account_to_receive, id) => {
        let mut balance_reserved = Amount::zero();
        let mut requests_active: BTreeMap<RequestId, Request> = BTreeMap::new();
        for(y, q) in host.state().requests.iter() {
            if q.expiry > now{
                requests_active.insert(*y, q.clone());
                balance_reserved += q.amount;
            }
        }
        // Keep the request still alive or not yet expired
        host.state_mut().requests = host.state_builder().new_map();
        for (b, e) in requests_active.iter(){
            host.state_mut().requests.insert(*b, e.clone());
        }

        let mut has = false;
        for (p, _) in host.state().requests.iter(){
            if *p == id{
                has = true;
                break;
            }
        }
        ensure!(!has, ErrorOnReceive::AlreadyExists);

        let balance = amount + host.self_balance();
        ensure!(
            balance >= transfer_amount,
            ErrorOnReceive::InsufficientBalance
        );

        let mut in_favour = BTreeSet::new();
        in_favour.insert(valid_address);

        let new_request = Request{
            amount: transfer_amount,
            sender_account: valid_address,
            accounts_in_aggrement: in_favour,
            receiver: account_to_receive,
            expiry: now.checked_add(host.state().init_params.timeout).ok_or(ErrorOnReceive::Overflow)?
        };

        host.state_mut().requests.insert(id, new_request);
        Ok(())
    }

    RequestAction::AcceptTransfer(amut, recipient_account, idd) => {
        let min_required_support = host.state().init_params.min_signers_req;

        let receipt = {
            let mut match_req = host.state_mut().requests.entry(idd).occupied_or(ErrorOnReceive::ParseParams)?;
            ensure!(match_req.expiry > now, ErrorOnReceive::TimedOut);
            ensure!(!match_req.accounts_in_aggrement.contains(&valid_address), ErrorOnReceive::AlreadyExists);

            match_req.accounts_in_aggrement.insert(valid_address);
            ensure!(match_req.accounts_in_aggrement.len() >= usize::from(min_required_support), ErrorOnReceive::Overflow);
        };
        if receipt == () {
            host.invoke_transfer(&recipient_account, amut)?;
            host.state_mut().requests.remove(&idd);
        }
        Ok(())
    }
}

}
