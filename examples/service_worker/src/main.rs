#![allow(opaque_hidden_inferred_bound)]

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{fmt::Subscriber, EnvFilter};
use warp::{http::StatusCode, *};
use web_push::{
    ContentEncoding, SubscriptionInfo, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

// An in-memory database
pub type Db = Arc<Mutex<Option<web_push::SubscriptionInfo>>>;

// Initialize an empty in-memory database.
#[must_use]
pub fn blank_db() -> Db {
    Arc::new(Mutex::new(None))
}

// Create a filter the includes the db.
fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || Arc::clone(&db))
}

// Create a "save_subscription" endpoint that accepts the subscription in json format.
pub fn save_subscription(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("save_subscription")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(save_subscription_handler)
}

// Save the subscription into the in-memory db.
async fn save_subscription_handler(
    subscription: SubscriptionInfo,
    db: Db,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    tracing::debug!("save_subscription: {:?}", subscription);

    let mut cur_subscription = db.lock().await;

    *cur_subscription = Some(subscription);

    Ok(StatusCode::CREATED)
}

// Create a "send_notification" endpoint. Pass the in-memory db to the handler.
pub fn send_notification(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("send_notification")
        .and(warp::post())
        .and(with_db(db))
        .and_then(send_notification_handler)
}

// The send_notification handler. When the endpoint is called, it should do the following:
// 1. Grab the subscription from the in-memory db.
// 2. Create a WebPushMessageBuilder from the subscription.
// 3. Set the payload.
// 4. Load the private key and build the vapid signature.
// 5. Create a new WebPush client.
// 6. Send the push message.
async fn send_notification_handler(db: Db) -> Result<impl warp::Reply, std::convert::Infallible> {
    let subscription = db.lock().await;
    if let Some(subscription) = subscription.as_ref() {
        tracing::debug!("subscription: {:?}", subscription);
        let mut builder = WebPushMessageBuilder::new(subscription);

        builder.set_payload(
            ContentEncoding::Aes128Gcm,
            "This is a push message coming from the server!".as_bytes(),
        );

        let private_key = tokio::fs::File::open("private_key.pem")
            .await
            .expect("open private key");
        let sig_builder =
            VapidSignatureBuilder::from_pem(private_key.into_std().await, subscription)
                .expect("create vapid signature");
        let signature = sig_builder.build().unwrap();
        builder.set_vapid_signature(signature);

        let client = web_push::IsahcWebPushClient::new().unwrap();

        client
            .send(builder.build().expect("build web push builder"))
            .await
            .expect("send push message");

        Ok(StatusCode::OK)
    } else {
        tracing::error!(
            "'send_notification' endpoint requested before a subscription was recorded."
        );
        Ok(StatusCode::BAD_REQUEST)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let db = blank_db();

    let cors = warp::cors()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_any_origin();

    let api = save_subscription(Arc::clone(&db))
        .or(send_notification(Arc::clone(&db)))
        .with(cors)
        .with(warp::log("subscription"));

    warp::serve(api).run(([127, 0, 0, 1], 8001)).await;

    Ok(())
}
