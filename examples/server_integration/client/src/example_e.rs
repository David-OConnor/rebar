use gloo_console::log;
use gloo_net::http::Request;
use seed::{prelude::*, *};
use std::mem;
use web_sys::{
    self,
    console::{log_1, log_2},
    File, FormData,
};

pub const TITLE: &str = "Example E";
pub const DESCRIPTION: &str =
    "Fill form and click 'Submit` button. Server echoes the form back. See console log for more info.";

type FetchResult<T> = Result<T, gloo_net::Error>;

const fn get_request_url() -> &'static str {
    "/api/form"
}

// ------ ------
//     Model
// ------ ------

#[derive(Default, Debug)]
pub struct Form {
    title: String,
    description: String,
    file: Option<File>,
    answer: bool,
}

impl Form {
    fn to_form_data(&self) -> Result<web_sys::FormData, JsValue> {
        let form_data = web_sys::FormData::new()?;
        form_data.append_with_str("title", &self.title)?;
        form_data.append_with_str("description", &self.description)?;
        if let Some(file) = &self.file {
            form_data.append_with_blob("file", file)?;
        }
        form_data.append_with_str("answer", &self.answer.to_string())?;
        Ok(form_data)
    }
}

pub enum Model {
    ReadyToSubmit(Form),
    WaitingForResponse(Form),
}

impl Default for Model {
    fn default() -> Self {
        Self::ReadyToSubmit(Form {
            title: "I'm title".into(),
            description: "I'm description".into(),
            file: None,
            answer: true,
        })
    }
}

impl Model {
    const fn form(&self) -> &Form {
        match self {
            Self::ReadyToSubmit(form) | Self::WaitingForResponse(form) => form,
        }
    }
    fn form_mut(&mut self) -> &mut Form {
        match self {
            Self::ReadyToSubmit(form) | Self::WaitingForResponse(form) => form,
        }
    }
}

// ------ ------
//    Update
// ------ ------

pub enum Msg {
    TitleChanged(String),
    DescriptionChanged(String),
    FileChanged(Option<File>),
    AnswerChanged,
    FormSubmitted(String),
    ServerResponded(FetchResult<String>),
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::TitleChanged(title) => model.form_mut().title = title,
        Msg::DescriptionChanged(description) => model.form_mut().description = description,
        Msg::FileChanged(file) => {
            model.form_mut().file = file;
        }
        Msg::AnswerChanged => toggle(&mut model.form_mut().answer),
        Msg::FormSubmitted(id) => {
            let form = mem::take(model.form_mut());
            let form_data = form.to_form_data().expect("create from data from form");
            orders.perform_cmd(async { Msg::ServerResponded(send_request(form_data).await) });
            *model = Model::WaitingForResponse(form);
            log!(format!("Form {id} submitted."));
        }
        Msg::ServerResponded(Ok(response_data)) => {
            *model = Model::ReadyToSubmit(Form::default());
            clear_file_input();
            log_2(
                &"%cResponse data:".into(),
                &"background: yellow; color: black".into(),
            );
            log_1(&response_data.into());
        }
        Msg::ServerResponded(Err(fetch_error)) => {
            *model = Model::ReadyToSubmit(mem::take(model.form_mut()));
            gloo_console::error!(format!("Request failed! {fetch_error:?}"));
        }
    }
}

async fn send_request(form: FormData) -> FetchResult<String> {
    Request::post(get_request_url())
        .body(JsValue::from(form))
        .send()
        .await?
        .text()
        .await
}

#[allow(clippy::option_map_unit_fn)]
fn clear_file_input() {
    seed::document()
        .get_element_by_id("form-file")
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|file_input| {
            // Note: `file_input.set_files(None)` doesn't work
            file_input.set_value("");
        });
}

fn toggle(value: &mut bool) {
    *value = !*value;
}

// ------ ------
//     View
// ------ ------

fn view_form_field(mut label: Node<Msg>, control: Node<Msg>) -> Node<Msg> {
    label.add_style("margin-right", unit!(7, px));
    div![
        style! {
          "margin-bottom" => unit!(7, px),
          "display" => "flex",
        },
        label,
        control
    ]
}

pub fn view(model: &Model, intro: impl FnOnce(&str, &str) -> Vec<Node<Msg>>) -> Vec<Node<Msg>> {
    let btn_enabled = matches!(model, Model::ReadyToSubmit(form) if !form.title.is_empty());

    let form_id = "A_FORM".to_string();
    let form = form![
        style! {
            St::Display => "flex",
            St::FlexDirection => "column",
        },
        ev(Ev::Submit, move |event| {
            event.prevent_default();
            Msg::FormSubmitted(form_id)
        }),
        view_form_field(
            label!["Title:", attrs! {At::For => "form-title" }],
            input![
                input_ev(Ev::Input, Msg::TitleChanged),
                attrs! {
                    At::Id => "form-title",
                    At::Value => model.form().title,
                    At::Required => true.as_at_value(),
                }
            ]
        ),
        view_form_field(
            label!["Description:", attrs! {At::For => "form-description" }],
            textarea![
                input_ev(Ev::Input, Msg::DescriptionChanged),
                attrs! {
                    At::Id => "form-description",
                    At::Value => model.form().description,
                    At::Rows => 1,
                },
            ],
        ),
        view_form_field(
            label!["Text file:", attrs! {At::For => "form-file" }],
            input![
                ev(Ev::Change, |event| {
                    let file = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
                        .and_then(|file_input| file_input.files())
                        .and_then(|file_list| file_list.get(0));

                    Msg::FileChanged(file)
                }),
                attrs! {
                    At::Type => "file",
                    At::Id => "form-file",
                    At::Accept => "text/plain",
                }
            ],
        ),
        view_form_field(
            label!["Do you like cocoa?:", attrs! {At::For => "form-answer" }],
            input![
                ev(Ev::Click, |_| Msg::AnswerChanged),
                attrs! {
                    At::Type => "checkbox",
                    At::Id => "form-answer",
                    At::Checked => model.form().answer.as_at_value(),
                }
            ],
        ),
        button![
            style! {
                "padding" => format!("{} {}", px(2), px(12)),
                "background-color" => if btn_enabled { CSSValue::from("aquamarine") } else { CSSValue::Ignored },
            },
            attrs! {At::Disabled => not(btn_enabled).as_at_value()},
            "Submit"
        ]
    ];

    nodes![intro(TITLE, DESCRIPTION), form]
}
