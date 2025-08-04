use yew::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlInputElement, HtmlSelectElement};
use crate::models::{AccountDetails, ClaimConfig, TransferConfig};
use crate::services::pi_api::{get_account_details, start_claim_worker, start_transfer_worker};

pub enum Msg {
    FetchDetails,
    UpdateDetails(AccountDetails),
    StartWorkers,
    UpdateWithdrawAddr(String),
    UpdateAmount(String),
    UpdateSelectedLock(String),
    UpdateSponsorSeed(String),
    UpdateFee(String),
    Log(String),
}

pub struct WithdrawPage {
    details: Option<AccountDetails>,
    logs: Vec<String>,
    withdraw_addr: String,
    amount: String,
    selected_lock: String,
    sponsor_seed: String,
    fee: String,
}

impl Component for WithdrawPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::FetchDetails);
        Self {
            details: None,
            logs: vec!["Welcome to the Control Panel. Enter details and start workers.".to_string()],
            withdraw_addr: "".to_string(),
            amount: "".to_string(),
            selected_lock: "".to_string(),
            sponsor_seed: "".to_string(),
            fee: "3200000".to_string(), // Default fee
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchDetails => {
                let link = ctx.link().clone();
                if let Ok(seed) = LocalStorage::get("pi_seed_phrase") {
                    wasm_bindgen_futures::spawn_local(async move {
                        match get_account_details(seed).await {
                            Ok(details) => link.send_message(Msg::UpdateDetails(details)),
                            Err(e) => link.send_message(Msg::Log(format!("Error fetching details: {}", e))),
                        }
                    });
                } else {
                    self.logs.push("Please login first.".to_string());
                }
                true
            }
            Msg::UpdateDetails(details) => {
                self.details = Some(details);
                true
            }
            Msg::StartWorkers => {
                if let (Ok(main_seed), Some(details)) = (LocalStorage::get("pi_seed_phrase"), &self.details) {
                    let link = ctx.link().clone();

                    // --- Transfer Worker ---
                    let transfer_config = TransferConfig {
                        seed_phrase: main_seed.clone(),
                        recipient_address: self.withdraw_addr.clone(),
                        amount: self.amount.clone(),
                        fee: self.fee.parse().unwrap_or(100000),
                    };
                    start_transfer_worker(transfer_config, link.callback(Msg::Log));

                    // --- Claim Worker ---
                    if !self.sponsor_seed.is_empty() && !self.selected_lock.is_empty() {
                         let claim_config = ClaimConfig {
                            main_wallet_seed: main_seed,
                            sponsor_wallet_seed: self.sponsor_seed.clone(),
                            lock_id: self.selected_lock.parse().unwrap_or(0),
                        };
                        start_claim_worker(claim_config, link.callback(Msg::Log));
                    } else {
                        link.send_message(Msg::Log("Sponsor seed and locked balance must be selected to start claim worker.".to_string()));
                    }

                } else {
                    ctx.link().send_message(Msg::Log("Cannot start workers. Login first.".to_string()));
                }
                false
            }
            Msg::Log(log) => {
                self.logs.push(log);
                if self.logs.len() > 100 { // Keep log size manageable
                    self.logs.remove(0);
                }
                true
            }
            Msg::UpdateWithdrawAddr(val) => { self.withdraw_addr = val; false }
            Msg::UpdateAmount(val) => { self.amount = val; false }
            Msg::UpdateSelectedLock(val) => { self.selected_lock = val; false }
            Msg::UpdateSponsorSeed(val) => { self.sponsor_seed = val; false }
            Msg::UpdateFee(val) => { self.fee = val; false }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let details_html = if let Some(details) = &self.details {
            html! {
                <div class="info-box">
                    <h3>{ "Wallet Info" }</h3>
                    <p>{ format!("Address: {}", details.public_key) }</p>
                    <p>{ format!("Available Balance: {} PI", details.available_balance) }</p>
                    <p>{ format!("Number of Locked Balances: {}", details.locked_balances.len()) }</p>
                </div>
            }
        } else {
            html! { <p>{ "Fetching wallet details..." }</p> }
        };

        let locked_balances_dropdown = if let Some(details) = &self.details {
            html! {
                <select onchange={link.callback(|e: Event| Msg::UpdateSelectedLock(e.target_unchecked_into::<HtmlSelectElement>().value()))}>
                    <option value="" disabled=true selected=true>{"Select a locked balance"}</option>
                    { for details.locked_balances.iter().map(|lb| html! { <option value={lb.id.to_string()}>{format!("{} PI unlocks at {}", lb.amount, lb.unlock_date)}</option> }) }
                </select>
            }
        } else {
            html! { <select disabled=true><option>{"Loading..."}</option></select> }
        };

        html! {
            <div>
                {details_html}

                <div class="form-group">
                    <label for="withdraw-addr">{ "Withdrawal Address" }</label>
                    <input type="text" id="withdraw-addr" onchange={link.callback(|e: Event| Msg::UpdateWithdrawAddr(e.target_unchecked_into::<HtmlInputElement>().value()))} />
                </div>

                <div class="form-group">
                    <label for="amount">{ "Amount to Transfer" }</label>
                    <input type="text" id="amount" placeholder="e.g., 100.0" onchange={link.callback(|e: Event| Msg::UpdateAmount(e.target_unchecked_into::<HtmlInputElement>().value()))} />
                </div>

                <div class="form-group">
                    <label for="fee">{ "Transfer Fee (in Stroops)" }</label>
                    <input type="text" id="fee" value={self.fee.clone()} onchange={link.callback(|e: Event| Msg::UpdateFee(e.target_unchecked_into::<HtmlInputElement>().value()))} />
                </div>

                <div class="form-group">
                    <label for="locked-balance">{ "Locked Balance to Claim" }</label>
                    {locked_balances_dropdown}
                </div>

                <div class="form-group">
                    <label for="sponsor-seed">{ "Sponsor Seed Phrase (for Claiming Fee)" }</label>
                    <input type="password" id="sponsor-seed" placeholder="Sponsor's secret seed phrase" onchange={link.callback(|e: Event| Msg::UpdateSponsorSeed(e.target_unchecked_into::<HtmlInputElement>().value()))} />
                </div>

                <button class="btn" onclick={link.callback(|_| Msg::StartWorkers)}>{ "START BOT" }</button>

                <div class="log-container">
                    { for self.logs.iter().map(|log| html! { <div class="log-entry">{log}</div> }) }
                </div>
            </div>
        }
    }
}