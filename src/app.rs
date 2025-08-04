use yew::prelude::*;
use chrono::{DateTime, Local};
use gloo_timers::callback::Interval;

use crate::components::login_page::LoginPage;
use crate::components::withdraw_page::WithdrawPage;

pub enum Page {
    Login,
    Withdraw,
}

pub enum Msg {
    TogglePage(Page),
    UpdateTime,
}

pub struct App {
    current_page: Page,
    server_time: DateTime<Local>,
    _interval: Interval,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let clock_handle = {
            let link = ctx.link().clone();
            Interval::new(1000, move || link.send_message(Msg::UpdateTime))
        };

        Self {
            current_page: Page::Login,
            server_time: Local::now(),
            _interval: clock_handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TogglePage(page) => {
                self.current_page = page;
                true
            }
            Msg::UpdateTime => {
                self.server_time = Local::now();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let login_active = matches!(self.current_page, Page::Login);
        let withdraw_active = matches!(self.current_page, Page::Withdraw);

        html! {
            <div class="container">
                <header>
                    <h1>{ "Pi Sweeper Bot Pro" }</h1>
                    <div id="server-time">{ self.server_time.format("%Y-%m-%d %H:%M:%S").to_string() }</div>
                </header>
                <div class="main-content">
                    <div class="toggle-buttons">
                        <button class={if login_active { "active" } else { "" }}
                                onclick={link.callback(|_| Msg::TogglePage(Page::Login))}>
                            { "Login" }
                        </button>
                        <button class={if withdraw_active { "active" } else { "" }}
                                onclick={link.callback(|_| Msg::TogglePage(Page::Withdraw))}>
                            { "Control Panel" }
                        </button>
                    </div>

                    {
                        match self.current_page {
                            Page::Login => html! { <LoginPage /> },
                            Page::Withdraw => html! { <WithdrawPage /> },
                        }
                    }
                </div>
            </div>
        }
    }
}

// This is needed to run the Yew application
fn main() {
    yew::Renderer::<App>::new().render();
}