use yew::prelude::*;
use web_sys::HtmlInputElement;
use gloo_storage::{LocalStorage, Storage};

pub enum Msg {
    UpdateSeed,
    Login,
}

#[derive(Properties, PartialEq)]
pub struct LoginPageProps {
    // In a real app, you'd have a callback to notify the parent of a successful login
}

pub struct LoginPage {
    seed_ref: NodeRef,
}

impl Component for LoginPage {
    type Message = Msg;
    type Properties = LoginPageProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            seed_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateSeed => false, // No re-render needed for input change
            Msg::Login => {
                if let Some(input) = self.seed_ref.cast::<HtmlInputElement>() {
                    let seed = input.value();
                    if !seed.trim().is_empty() {
                        // Store the seed phrase in local storage for the session
                        LocalStorage::set("pi_seed_phrase", seed).expect("Failed to set seed in local storage");
                        gloo_console::log!("Seed phrase stored. Navigate to Control Panel.");
                        // In a more complex app, you would navigate programmatically.
                        // Here, we rely on the user to click the tab.
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <h2>{ "Login" }</h2>
                <p>{ "Enter your wallet's seed phrase to begin." }</p>
                <div class="form-group">
                    <label for="seed">{ "Seed Phrase" }</label>
                    <input type="password" id="seed" placeholder="Your secret seed phrase" ref={self.seed_ref.clone()} onchange={ctx.link().callback(|_| Msg::UpdateSeed)} />
                </div>
                <button class="btn" onclick={ctx.link().callback(|_| Msg::Login)}>{ "Login & Save" }</button>
            </div>
        }
    }
}