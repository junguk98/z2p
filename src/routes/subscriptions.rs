use actix_web::{web, HttpResponse};
use actix_web::web::Data;
use chrono::Utc;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;
use crate::utils::Z2pError;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(value.email)?;
        let name = SubscriberName::parse(value.name)?;

        Ok(NewSubscriber { email, name })
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
name = "Adding a new subscriber",
skip(form, pool, email_client, base_url),
fields(
subscriber_email = % form.email,
subscriber_name = % form.name
)
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    match handle_subscribe(form, &pool, &email_client, &base_url).await {
        Err(e) => match e.downcast_ref() {
            Some(Z2pError::BadRequest) => HttpResponse::BadRequest().finish(),
            _ => HttpResponse::InternalServerError().finish()
        }
        Ok(()) => HttpResponse::Ok().finish()
    }
}

async fn handle_subscribe(
    form: web::Form<FormData>,
    pool: &web::Data<PgPool>,
    email_client: &web::Data<EmailClient>,
    base_url: &web::Data<ApplicationBaseUrl>,
) -> anyhow::Result<()> {
    let new_subscriber = match form.0.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => {
            return Err(anyhow::anyhow!(Z2pError::BadRequest));
        }
    };
    let mut transaction = pool.begin().await?;
    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber).await?;
    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token).await?;
    transaction.commit().await?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token)
        .await?;

    Ok(())
}

#[tracing::instrument(
name = "Saving new subscriber details in the database",
skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(transaction: &mut Transaction<'_, Postgres>, new_subscriber: &NewSubscriber) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, $5)
            "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
        "pending_confirmation"
    );

    transaction.execute(query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(subscriber_id)
}

async fn send_confirmation_email(
    email_client: &Data<EmailClient>,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url,
        subscription_token
    );

    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!", &html_body, &plain_body,
        ).await
}

#[tracing::instrument(
name = "Store subscription token in the database",
skip(subscriber_id, subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    );
    transaction.execute(query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}