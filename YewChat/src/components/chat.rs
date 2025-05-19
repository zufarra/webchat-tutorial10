use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
    let submit = ctx.link().callback(|_| Msg::SubmitMessage);
    html! {
        <div class="flex w-screen bg-gradient-to-br from-purple-100 to-blue-100 min-h-screen">
            <div class="flex-none w-56 h-screen bg-white shadow-lg">
                <div class="text-xl p-3 font-bold text-blue-700">{"👥 Users"}</div>
                {
                    self.users.clone().iter().map(|u| {
                        html!{
                            <div class="flex m-3 bg-blue-50 rounded-lg p-2 shadow-sm">
                                <img class="w-10 h-10 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                <div class="ml-3">
                                    <div class="text-sm font-semibold">{u.name.clone()}</div>
                                    <div class="text-xs text-gray-500">{"Hi there!"}</div>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
            <div class="grow h-screen flex flex-col">
                <div class="w-full h-14 border-b-2 border-gray-300 flex items-center px-4 bg-white shadow-sm">
                    <div class="text-xl font-semibold text-blue-800">{"💬 Creative Chat"}</div>
                </div>
                <div class="w-full grow overflow-auto px-6 py-4 space-y-4">
                    {
                        self.messages.iter().map(|m| {
                            let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                            let is_me = m.from == "me"; // Ganti dengan user saat ini kalau perlu
                            let bubble_align = if is_me { "justify-end" } else { "justify-start" };
                            let bubble_color = if is_me { "bg-blue-200 text-right rounded-tl-lg rounded-bl-lg rounded-br-lg" } else { "bg-white text-left rounded-tr-lg rounded-br-lg rounded-bl-lg" };

                            let content = if m.message.ends_with(".gif") {
                                html! { <img class="mt-2 rounded-lg max-w-xs" src={m.message.clone()} /> }
                            } else {
                                let replaced = m.message.replace(":smile:", "😊").replace(":love:", "❤️");
                                html! { <div class="text-sm text-gray-800">{replaced}</div> }
                            };

                            html!{
                                <div class={classes!("flex", bubble_align)}>
                                    <div class={classes!("max-w-md", "flex", "items-end", bubble_color, "p-4", "shadow", "space-x-2")}>
                                        <img class="w-8 h-8 rounded-full" src={user.avatar.clone()} alt="avatar"/>
                                        <div>
                                            <div class="text-xs font-bold text-gray-600">{m.from.clone()}</div>
                                            { content }
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="w-full h-16 bg-white shadow-inner flex items-center px-4">
                    <input
                        ref={self.chat_input.clone()}
                        type="text"
                        placeholder="Type a message or send a .gif..."
                        class="block w-full py-2 pl-4 mx-3 bg-gray-100 rounded-full outline-none focus:text-gray-700 focus:ring-2 focus:ring-blue-300"
                        name="message"
                        required=true
                    />
                    <button
                        onclick={submit}
                        class="p-3 bg-blue-600 hover:bg-blue-700 transition rounded-full flex justify-center items-center text-white"
                    >
                        <svg fill="currentColor" viewBox="0 0 24 24" class="w-5 h-5">
                            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
}
}