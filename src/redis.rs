use crate::arena::ArenaFull;
use crate::repo::Repo;
use futures::stream::StreamExt;
use redis::RedisError;
use serde_json::Error as SerdeJsonError;
use std::sync::Arc;
use thiserror::Error as ThisError;
use tracing::error;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("[REDIS] Error getting payload: {0}")]
    RedisError(#[from] RedisError),
    #[error("[SERDE] Error parsing JSON: {0}")]
    SerdeJsonError(#[from] SerdeJsonError),
}

pub fn parse_message(msg: &redis::Msg) -> Result<ArenaFull, Error> {
    Ok(serde_json::from_str(&msg.get_payload::<String>()?)?)
}

pub fn subscribe(opt: crate::opt::Opt, repo: Arc<Repo>) -> Result<(), Error> {
    let _ = tokio::spawn(async move {
        let client = redis::Client::open(opt.redis_url).unwrap();
        let subscribe_con = client.get_tokio_connection().await.unwrap();
        let mut pubsub = subscribe_con.into_pubsub();
        pubsub.subscribe("http-out").await.unwrap();
        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            parse_message(&msg)
                .map_err(|e| error!("{:?}", e))
                .map(|full| async { repo.put(full).await })
                .ok();
        }
    });
    Ok(())
}
