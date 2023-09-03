use crate::js_sys::Uint8Array;
use gloo_console::log;
use gloo_net::http::Request;
use seed::{prelude::*, *};
use shared::{
    decrypt, encrypt,
    opaque_ke::{
        keypair::Key,
        opaque::{ClientLogin, ClientRegistration},
    },
    rand_core::OsRng,
    DefaultCipherSuite,
};

type FetchResult<T> = Result<T, gloo_net::Error>;

// ------ ------
//     Init
// ------ ------

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async {
        Msg::PublicKeyFetched(
            async { Request::post("api/public-key").send().await?.binary().await }.await,
        )
    });

    Model {
        public_key: None,
        rng: OsRng,
        registration_password: "pass".to_owned(),
        registration_state: None,
        registration_status: "Not registered".to_owned(),
        login_password: "pass".to_owned(),
        login_state: None,
        login_status: "Not logged in".to_owned(),
        message_to_send: "Hello!".to_owned(),
        received_message: None,
        shared_secret: None,
    }
}

// ------ ------
//     Model
// ------ ------

struct Model {
    public_key: Option<Key>,
    rng: OsRng,
    registration_password: String,
    registration_state: Option<ClientRegistration<DefaultCipherSuite>>,
    registration_status: String,
    login_password: String,
    login_state: Option<ClientLogin<DefaultCipherSuite>>,
    login_status: String,
    message_to_send: String,
    received_message: Option<String>,
    shared_secret: Option<Vec<u8>>,
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    PublicKeyFetched(FetchResult<Vec<u8>>),
    RegistrationPasswordChanged(String),
    RegisterStep1,
    RegisterStep2(FetchResult<Vec<u8>>),
    Registered(FetchResult<String>),
    LoginPasswordChanged(String),
    LoginStep1,
    LoginStep2(FetchResult<Vec<u8>>),
    LoggedIn(FetchResult<String>),
    MessageToSendChanged(String),
    SendMessage,
    MessageReceived(FetchResult<Vec<u8>>),
}

#[allow(clippy::too_many_lines)]
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::PublicKeyFetched(key) => {
            let key = key
                .expect("public key bytes")
                .as_slice()
                .try_into()
                .expect("public key");
            model.public_key = Some(key);
        }
        Msg::RegistrationPasswordChanged(message) => {
            model.registration_password = message;
        }
        Msg::RegisterStep1 => {
            let (r1, state) = ClientRegistration::<DefaultCipherSuite>::start(
                model.registration_password.as_bytes(),
                Some(b"pepper"),
                &mut model.rng,
            )
            .expect("reg step 1");

            model.registration_state = Some(state);

            let request = Request::post("api/registration/step-1").binary(r1.to_bytes());
            orders.perform_cmd(async {
                Msg::RegisterStep2(async { request.send().await?.binary().await }.await)
            });
        }
        Msg::RegisterStep2(r2) => {
            let (r3, _) = model
                .registration_state
                .take()
                .expect("registration state")
                .finish(
                    r2.expect("r2 bytes").as_slice().try_into().expect("r2"),
                    model.public_key.as_ref().expect("public key reference"),
                    &mut model.rng,
                )
                .expect("reg step 2");

            let request = Request::post("api/registration/step-2").binary(r3.to_bytes());
            orders.perform_cmd(async {
                Msg::Registered(async { request.send().await?.text().await }.await)
            });
        }
        Msg::Registered(status) => {
            model.registration_status = status.expect("registration status");
        }
        Msg::LoginPasswordChanged(message) => {
            model.login_password = message;
        }
        Msg::LoginStep1 => {
            let (l1, state) = ClientLogin::<DefaultCipherSuite>::start(
                model.login_password.as_bytes(),
                Some(b"pepper"),
                &mut model.rng,
            )
            .expect("login step 1");

            model.login_state = Some(state);

            let request = Request::post("api/login/step-1").binary(l1.to_bytes());
            orders.perform_cmd(async {
                Msg::LoginStep2(async { request.send().await?.binary().await }.await)
            });
        }
        Msg::LoginStep2(l2) => {
            let (l3, shared_secret, _) = model
                .login_state
                .take()
                .expect("registration state")
                .finish(
                    l2.expect("l2 bytes").as_slice().try_into().expect("l2"),
                    model.public_key.as_ref().expect("public key reference"),
                    &mut model.rng,
                )
                .expect("login step 2");

            log!(format!("Shared secret: {shared_secret:?}"));
            model.shared_secret = Some(shared_secret);

            let request = Request::post("api/login/step-2").binary(l3.to_bytes());
            orders.perform_cmd(async {
                Msg::LoggedIn(async { request.send().await?.text().await }.await)
            });
        }
        Msg::LoggedIn(status) => {
            model.login_status = status.expect("login status");
        }
        Msg::MessageToSendChanged(message) => {
            model.message_to_send = message;
        }
        Msg::SendMessage => {
            let request = Request::post("api/echo").binary(encrypt(
                model.message_to_send.as_bytes(),
                model.shared_secret.as_ref().expect("shared key"),
            ));

            orders.perform_cmd(async {
                Msg::MessageReceived(async { request.send().await?.binary().await }.await)
            });
        }
        Msg::MessageReceived(message) => {
            let message = decrypt(
                &message.expect("encrypted echoed message"),
                model.shared_secret.as_ref().expect("shared key"),
            );
            let message = String::from_utf8(message).expect("echoed message");
            model.received_message = Some(message);
        }
    }
}

trait BinaryBody {
    fn binary(self, bytes: impl AsRef<[u8]>) -> Self;
}

impl BinaryBody for Request {
    fn binary(self, bytes: impl AsRef<[u8]>) -> Self {
        let bytes = Uint8Array::from(bytes.as_ref());
        self.body(bytes)
            .header("Content-Type", "application/octet-stream")
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    div![
        div![
            "Public key fetched: ",
            model.public_key.is_some().to_string(),
        ],
        div![
            input![
                attrs! {
                    At::Value => model.registration_password,
                },
                input_ev(Ev::Input, Msg::RegistrationPasswordChanged),
            ],
            button!["Register", ev(Ev::Click, |_| Msg::RegisterStep1),]
        ],
        div!["Registration status: ", &model.registration_status,],
        div![
            input![
                attrs! {
                    At::Value => model.login_password,
                },
                input_ev(Ev::Input, Msg::LoginPasswordChanged),
            ],
            button!["Login", ev(Ev::Click, |_| Msg::LoginStep1),]
        ],
        div!["Login status: ", &model.login_status,],
        div![
            input![
                attrs! {
                    At::Value => model.message_to_send,
                },
                input_ev(Ev::Input, Msg::MessageToSendChanged),
            ],
            button!["Send", ev(Ev::Click, |_| Msg::SendMessage),]
        ],
        div!["Received message: ", &model.received_message,]
    ]
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
