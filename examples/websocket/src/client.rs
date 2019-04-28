#[macro_use]
extern crate seed;
#[macro_use]
extern crate serde_derive;

use seed::{prelude::*, App};
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

mod json;

const WS_URL: &str = "ws://127.0.0.1:9000/ws";

#[derive(Clone, Default)]
struct Model {
    connected: bool,
    msg_rx_cnt: usize,
    msg_tx_cnt: usize,
    input_text: String,
    messages: Vec<String>,
}

#[derive(Clone)]
enum Msg {
    Connected,
    ServerMsg(json::ServerMsg),
    Send(json::ClientMsg),
    Sent,
    EditChange(String),
}

fn update(msg: Msg, mut model: &mut Model) -> Update<Msg> {
    match msg {
        Msg::Connected => {
            model.connected = true;
            Render.into()
        }
        Msg::ServerMsg(msg) => {
            model.connected = true;
            model.msg_rx_cnt += 1;
            model.messages.push(msg.text);
            Render.into()
        }
        Msg::EditChange(input_text) => {
            model.input_text = input_text;
            Render.into()
        }
        Msg::Send(_) => Skip.into(),
        Msg::Sent => {
            model.input_text = "".into();
            model.msg_tx_cnt += 1;
            Render.into()
        }
    }
}

fn render_messages(msgs: &[String]) -> El<Msg> {
    let msgs: Vec<_> = msgs.iter().map(|m| p![m]).collect();
    div![msgs]
}

fn view(model: &Model) -> Vec<El<Msg>> {
    vec![
        h1!["seed websocket example"],
        if model.connected {
            div![
                input![
                    attrs! {
                        "type"=>"text";
                        "id"=>"text";
                        At::Value => model.input_text;
                    },
                    input_ev(Ev::Input, Msg::EditChange)
                ],
                button![
                    attrs! {"type"=>"button";"id"=>"send"},
                    simple_ev(
                        "click",
                        Msg::Send(json::ClientMsg {
                            text: model.input_text.clone()
                        })
                    ),
                    "Send"
                ]
            ]
        } else {
            div![p![em!["Connecting..."]]]
        },
        render_messages(&model.messages),
        footer![
            if model.connected {
                p!["Connected"]
            } else {
                p!["Disconnected"]
            },
            p![format!("{} messages received", model.msg_rx_cnt)],
            p![format!("{} messages sent", model.msg_tx_cnt)]
        ],
    ]
}

#[wasm_bindgen]
pub fn start() {
    log!("Start the websocket client app");
    let app = App::build(Model::default(), update, view).finish().run();

    let ws = WebSocket::new(WS_URL).unwrap();
    register_handlers(&ws, app.clone());
    register_message_listener(ws, app)
}

fn register_handlers<ElC>(ws: &web_sys::WebSocket, app: App<Msg, Model, ElC>)
    where ElC: ElContainer<Msg> + 'static
{
    register_handler_on_open(&ws, app.clone());
    register_handler_on_message(&ws, app);
    register_handler_on_close(&ws);
    register_handler_on_error(&ws);
}

fn register_message_listener<ElC>(ws: web_sys::WebSocket, app: App<Msg, Model, ElC>)
    where ElC: ElContainer<Msg> + 'static
{
    app.clone().add_message_listener(move |msg| match msg {
        Msg::Send(msg) => {
            let s = serde_json::to_string(msg).unwrap();
            ws.send_with_str(&s).unwrap();
            app.update(Msg::Sent);
        }
        _ => {}
    });
}

// ------ HANDLERS -------

fn register_handler_on_open<ElC>(ws: &web_sys::WebSocket, app: App<Msg, Model, ElC>)
    where ElC: ElContainer<Msg> + 'static
{
    let on_open = Closure::wrap(Box::new(move |_| {
        log!("WebSocket connection is open now");
        app.update(Msg::Connected);
    }) as Box<FnMut(JsValue)>);

    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();
}

fn register_handler_on_close(ws: &web_sys::WebSocket) {
    let on_close = Closure::wrap(Box::new(|_| {
        log!("WebSocket connection was closed");
    }) as Box<FnMut(JsValue)>);

    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget();
}

fn register_handler_on_message<ElC>(ws: &web_sys::WebSocket, app: App<Msg, Model, ElC>)
    where ElC: ElContainer<Msg> + 'static
{
    let on_message = Closure::wrap(Box::new(move |ev: MessageEvent| {
        log!("Client received a message");
        let txt = ev.data().as_string().unwrap();
        let json: json::ServerMsg = serde_json::from_str(&txt).unwrap();
        log!("- text message: ", &txt);
        app.update(Msg::ServerMsg(json));
    }) as Box<FnMut(MessageEvent)>);

    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
}

fn register_handler_on_error(ws: &web_sys::WebSocket) {
    let on_error = Closure::wrap(Box::new(|_| {
        log!("Error");
    }) as Box<FnMut(JsValue)>);

    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();
}