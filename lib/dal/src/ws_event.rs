use crate::{BillingAccountId, ChangeSetPk, SchemaPk, Tenancy};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WsEventError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
}

pub type WsEventResult<T> = Result<T, WsEventError>;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "kind", content = "data")]
pub enum WsPayload {
    ChangeSetCreated(ChangeSetPk),
    ChangeSetApplied(ChangeSetPk),
    ChangeSetCanceled(ChangeSetPk),
    SchemaCreated(SchemaPk),
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct WsEvent {
    version: i64,
    billing_account_ids: Vec<BillingAccountId>,
    payload: WsPayload,
}

impl WsEvent {
    pub fn new(billing_account_ids: Vec<BillingAccountId>, payload: WsPayload) -> Self {
        WsEvent {
            version: 1,
            billing_account_ids,
            payload,
        }
    }

    pub fn billing_account_id_from_tenancy(tenancy: &Tenancy) -> Vec<BillingAccountId> {
        tenancy.billing_account_ids.clone()
    }

    pub async fn publish(&self, nats: &NatsTxn) -> WsEventResult<()> {
        for billing_account_id in self.billing_account_ids.iter() {
            let subject = format!(
                "si.billing_account_id.{}.event",
                billing_account_id.to_string()
            );
            nats.publish(subject, &self).await?;
        }
        Ok(())
    }
}
