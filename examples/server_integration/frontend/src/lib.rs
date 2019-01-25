#[macro_use]
extern crate seed;
use seed::prelude::*;
use seed::{spawn_local, Method, Request};

use futures::Future;

use shared::Data;

// Model

#[derive(Clone)]
struct Model {
    pub data: Data,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            data: Data {
                val: 0,
                text: "".into(),
            },
        }
    }
}

fn get_data(state: seed::App<Msg, Model>) -> impl Future<Item = (), Error = JsValue> {
    let url = "http://localhost:8001/data";

    Request::new(url)
        .method(Method::Get)
        .fetch_json()
        .map(move |json| {
            state.update(Msg::Replace(json));
        })
}

// Update

#[derive(Clone)]
enum Msg {
    GetData(seed::App<Msg, Model>),
    Replace(Data),
}

fn update(msg: Msg, model: Model) -> Update<Model> {
    match msg {
        Msg::GetData(state) => {
            spawn_local(get_data(state));
            Render(model)
        }
        Msg::Replace(data) => Render(Model { data }),
    }
}

// View

fn view(state: seed::App<Msg, Model>, model: &Model) -> El<Msg> {
    div![
        h1![format!("Val: {} Text: {}", model.data.val, model.data.text)],
        button![
            raw_ev("click", move |_| Msg::GetData(state.clone())),
            "Update data"
        ]
    ]
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(Model::default(), update, view)
        .mount("main")
        .finish()
        .run();
}
