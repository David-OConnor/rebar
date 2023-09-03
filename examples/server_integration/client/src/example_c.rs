use gloo_net::http::Request;
use seed::{prelude::*, *};
use std::rc::Rc;

pub const TITLE: &str = "Example C";
pub const DESCRIPTION: &str =
    "Click button 'Send request` to send request to endpoint with configurable delay.
    Click again to abort request.";

type FetchResult<T> = Result<T, gloo_net::Error>;

fn get_request_url() -> String {
    let response_delay_ms: u32 = 2000;
    format!("/api/delayed-response/{response_delay_ms}")
}

// ------ ------
//     Model
// ------ ------

pub struct Model {
    pub fetch_result: Option<FetchResult<String>>,
    pub abort_controller: Rc<web_sys::AbortController>,
    pub status: Status,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            fetch_result: None,
            abort_controller: Rc::new(web_sys::AbortController::new().unwrap()),
            status: Status::default(),
        }
    }
}

pub enum Status {
    ReadyToSendRequest,
    WaitingForResponse,
    RequestAborted,
}

impl Default for Status {
    fn default() -> Self {
        Self::ReadyToSendRequest
    }
}

// ------ ------
//    Update
// ------ ------

pub enum Msg {
    SendRequest,
    AbortRequest,
    Fetched(FetchResult<String>),
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SendRequest => {
            let abort_signal = model.abort_controller.signal();
            let request = Request::get(&get_request_url()).abort_signal(Some(&abort_signal));
            model.status = Status::WaitingForResponse;
            model.fetch_result = None;
            orders.perform_cmd(async {
                Msg::Fetched(async { request.send().await?.text().await }.await)
            });
        }

        Msg::AbortRequest => {
            model.abort_controller.abort();
            model.status = Status::RequestAborted;
        }

        Msg::Fetched(fetch_result) => {
            model.status = Status::ReadyToSendRequest;
            model.fetch_result = Some(fetch_result);
        }
    }
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model, intro: impl FnOnce(&str, &str) -> Vec<Node<Msg>>) -> Vec<Node<Msg>> {
    nodes![
        intro(TITLE, DESCRIPTION),
        match model.status {
            Status::ReadyToSendRequest => nodes![
                model
                    .fetch_result
                    .as_ref()
                    .map(|result| div![format!("{result:#?}")]),
                button![ev(Ev::Click, |_| Msg::SendRequest), "Send request"],
            ],
            Status::WaitingForResponse => nodes![
                div!["Waiting for response..."],
                button![ev(Ev::Click, |_| Msg::AbortRequest), "Abort request"],
            ],
            Status::RequestAborted => nodes![
                model
                    .fetch_result
                    .as_ref()
                    .map(|result| div![format!("{result:#?}")]),
                button![
                    attrs! {At::Disabled => false.as_at_value()},
                    "Request aborted"
                ],
            ],
        }
    ]
}
