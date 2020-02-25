//! Fetch POST example.

use seed::{prelude::*, *};

#[derive(serde::Serialize, Default)]
pub struct Form {
    name: String,
}

#[derive(Default)]
pub struct Model {
    form: Form,
}

pub enum Msg {
    NameChanged(String),
    Submit,
    Submited,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::NameChanged(name) => model.form.name = name,
        Msg::Submit => {
            orders.skip(); // No need to rerender

            let token = "YWxhZGRpbjpvcGVuc2VzYW1l";
            // Created outside async block because of lifetime reasons
            // (we can't use reference to `model.from` in async
            // function).
            let request = Request::new("/")
                .method(Method::Post)
                .header(Header::custom("Accept-Language", "en"))
                .header(Header::custom("Authorization", format!("Basic {}", token)))
                .json(&model.form)
                .expect("Serialization failed");

            orders.perform_cmd(async {
                let response = fetch(request).await.expect("HTTP request failed");
                assert!(response.status().is_ok());
                Msg::Submited
            });
        }
        Msg::Submited => {
            model.form.name = "".into();
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    form![
        ev(Ev::Submit, |_| Msg::Submit),
        label![
            "Name",
            input![
                attrs! {At::Value => model.form.name},
                input_ev(Ev::Input, Msg::NameChanged),
            ]
        ],
        button![
            "Submit",
            ev(Ev::Click, |event| {
                event.prevent_default();
                Msg::Submit
            })
        ]
    ]
}
