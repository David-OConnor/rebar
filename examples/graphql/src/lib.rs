#![allow(clippy::large_enum_variant, clippy::cognitive_complexity)]

use gloo_console::log;
use gloo_net::http::Request;
use graphql_client::{GraphQLQuery, Response as GQLResponse};
use itertools::Itertools;
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://countries.trevorblades.com/";

type Code = String;
type FetchResult<T> = Result<T, gloo_net::Error>;

// ------ ------
//    GraphQL
// ------ ------

macro_rules! generate_query {
    ($query:ident) => {
        #[derive(GraphQLQuery)]
        #[graphql(
            schema_path = "graphql/schema.graphql",
            query_path = "graphql/queries.graphql",
            response_derives = "Debug"
        )]
        struct $query;
    };
}
generate_query!(QContinents);
generate_query!(QContinent);
generate_query!(QCountry);

async fn send_graphql_request<V, T>(variables: &V) -> FetchResult<T>
where
    V: Serialize,
    T: for<'de> Deserialize<'de> + 'static,
{
    Request::post(API_URL)
        .json(variables)?
        .send()
        .await?
        .json()
        .await
}

// ------ ------
//     Init
// ------ ------

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async {
        Msg::ContinentsFetched(
            send_graphql_request(&QContinents::build_query(q_continents::Variables)).await,
        )
    });
    Model::default()
}

// ------ ------
//     Model
// ------ ------

#[derive(Default)]
struct Model {
    continents: Option<Vec<q_continents::QContinentsContinents>>,
    selected_continent: Option<Code>,
    countries: Option<Vec<q_continent::QContinentContinentCountries>>,
    selected_country: Option<Code>,
    country: Option<q_country::QCountryCountry>,
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    ContinentsFetched(FetchResult<GQLResponse<q_continents::ResponseData>>),
    ContinentClicked(Code),
    CountriesFetched(FetchResult<GQLResponse<q_continent::ResponseData>>),
    CountryClicked(Code),
    CountryFetched(FetchResult<GQLResponse<q_country::ResponseData>>),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ContinentsFetched(Ok(GQLResponse {
            data: Some(data), ..
        })) => {
            model.continents = Some(data.continents);
        }
        Msg::ContinentsFetched(error) => log!(format!("{error:?}")),
        Msg::ContinentClicked(code) => {
            model.selected_continent = Some(code.clone());
            orders.perform_cmd(async {
                Msg::CountriesFetched(
                    send_graphql_request(&QContinent::build_query(q_continent::Variables { code }))
                        .await,
                )
            });
        }
        Msg::CountriesFetched(Ok(GQLResponse {
            data: Some(data), ..
        })) => {
            model.countries = data.continent.map(|continent| continent.countries);
        }
        Msg::CountriesFetched(error) => log!(format!("{error:?}")),
        Msg::CountryClicked(code) => {
            model.selected_country = Some(code.clone());
            orders.perform_cmd(async {
                Msg::CountryFetched(
                    send_graphql_request(&QCountry::build_query(q_country::Variables { code }))
                        .await,
                )
            });
        }
        Msg::CountryFetched(Ok(GQLResponse {
            data: Some(data), ..
        })) => {
            model.country = data.country;
        }
        Msg::CountryFetched(error) => log!(format!("{error:?}")),
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    let continents = model.continents.as_ref().map(|continents| {
        continents
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
            .map(|continent| continent_row(continent, model.selected_continent.as_ref()))
            .collect::<Vec<_>>()
    });

    let countries = model.countries.as_ref().map(|countries| {
        countries
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
            .map(|country| country_row(country, model.selected_country.as_ref()))
            .collect::<Vec<_>>()
    });

    div![
        C!["container"],
        div![
            C!["columns"],
            column("Continents", continents),
            column("Countries", countries),
            column("Country", model.country.as_ref().map(country_detail)),
        ]
    ]
}

fn column(title: &str, content: impl IntoNodes<Msg>) -> Node<Msg> {
    div![
        C!["column"],
        div![
            C!["box"],
            div![
                C!["menu"],
                p![C!["menu-label"], title,],
                ul![
                    C!["menu-list", "content"],
                    style! {
                        St::MaxHeight => vh(80),
                        St::OverflowY => "auto",
                    },
                    content.into_nodes(),
                ]
            ],
        ]
    ]
}

fn continent_row(
    continent: &q_continents::QContinentsContinents,
    selected: Option<&Code>,
) -> Node<Msg> {
    let code = continent.code.clone();
    li![a![
        // @TODO use `Option::contains` once stable
        C![IF!(Some(&code) == selected => "is-active")],
        &continent.name,
        ev(Ev::Click, move |_| Msg::ContinentClicked(code)),
    ],]
}

fn country_row(
    country: &q_continent::QContinentContinentCountries,
    selected: Option<&Code>,
) -> Node<Msg> {
    let code = country.code.clone();
    li![a![
        // @TODO use `Option::contains` once stable
        C![IF!(Some(&code) == selected => "is-active")],
        &country.name,
        ev(Ev::Click, move |_| Msg::CountryClicked(code)),
    ],]
}

fn country_detail(country: &q_country::QCountryCountry) -> Vec<Node<Msg>> {
    nodes![
        p![C!["title", "is-5"], &country.name],
        p![
            "Code: ",
            span![C!["has-text-weight-semibold"], &country.code]
        ],
        p![
            "Native name: ",
            span![C!["has-text-weight-semibold"], &country.native]
        ],
        p![
            "Capital: ",
            span![C!["has-text-weight-semibold"], &country.capital]
        ],
        p![
            "Currency: ",
            span![C!["has-text-weight-semibold"], &country.currency]
        ],
        p![
            "Phone prefix: ",
            span![C!["has-text-weight-semibold"], &country.phone]
        ],
        p!["Languages: ", view_languages(&country.languages)],
        IF!(not(country.states.is_empty()) => p!["States: ", view_states(&country.states)]),
    ]
}

fn view_languages(languages: &[q_country::QCountryCountryLanguages]) -> Node<Msg> {
    ul![languages
        .iter()
        .map(|lang| { li![&lang.name, IF!(lang.rtl => " (RTL)"),] })]
}

fn view_states(states: &[q_country::QCountryCountryStates]) -> Node<Msg> {
    ul![states.iter().map(|state| li![&state.name])]
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
