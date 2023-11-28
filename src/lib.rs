#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_std::{collections::*, *};
use core::fmt::Debug;

#[derive(Serialize, SchemaType, Clone)]
struct InitialiseParams{
    timeout : Timestamp,

    #[concordium(size_length = 2)]
    signers: BTreeSet<AccountAddress>,

    min_signers_req: u16
}
/// Your smart contract state.
#[derive(Serial, SchemaType, DeserialWithState)]
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

pub enum ErrorOnReceive{
    NotReceived
}

/// Init function that creates a new smart contract.
#[init(contract = "multi_sig", parameter = "InitialiseParams", payable)]
fn init<St: HasStateApi>(_ctx: &InitContext, amount: Amount, _state_builder: &mut StateBuilder<St>) -> Result<State<St>, Error> {
    // Your code
    let parameter: InitialiseParams = match _ctx.parameter_cursor().get(){
        Ok(parameter) => parameter,
        Err(_) => return Err(Reject::default()),
    };
    ensure!(parameter.min_signers_req <= parameter.signers.len() as u16, Error::AccountsLessThanSupportNeeded);

    ensure!(parameter.signers.len() >= 3, Error::IncompleteAccounts);

    let state = State {
        init_params: parameter,
        requests: _state_builder.new_map(),
    };

    Ok(state)
}

/// Receive function. The input parameter is the boolean variable `throw_error`.
///  If `throw_error == true`, the receive function will throw a custom error.
///  If `throw_error == false`, the receive function executes successfully.
#[receive(
    contract = "multi_sig",
    name = "receive",
    error = "Error",
    payable
)]
#[inline(always)]
fn receive<St: HasStateApi>(ctx: &ReceiveContext, amount: Amount, _host: &mut Host<State<St>>) -> Result<(), Error> {
    // Your code
        Ok(())
}

/// View function that returns the content of the state.
#[receive(contract = "multi_sig", name = "view", return_value = "State")]
fn view<'b>(_ctx: &ReceiveContext, host: &'b Host<State>) -> ReceiveResult<&'b State> {
    Ok(host.state())
}
