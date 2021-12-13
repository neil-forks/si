use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    Group, GroupId, HistoryActor, HistoryEventError, StandardModel, StandardModelError, Tenancy,
    Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum CapabilityError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type CapabilityResult<T> = Result<T, CapabilityError>;

pk!(CapabilityPk);
pk!(CapabilityId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    pk: CapabilityPk,
    id: CapabilityId,
    subject: String,
    action: String,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Capability,
    pk: CapabilityPk,
    id: CapabilityId,
    table_name: "capabilities",
    history_event_label_base: "capability",
    history_event_message_name: "Capability"
}

impl Capability {
    #[tracing::instrument(skip(txn, nats, subject, action))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        subject: impl AsRef<str>,
        action: impl AsRef<str>,
    ) -> CapabilityResult<Self> {
        let subject = subject.as_ref();
        let action = action.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM capability_create_v1($1, $2, $3, $4)",
                &[tenancy, visibility, &subject, &action],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor!(subject, String, CapabilityResult);
    standard_model_accessor!(action, String, CapabilityResult);

    standard_model_belongs_to!(
        lookup_fn: group,
        set_fn: set_group,
        unset_fn: unset_group,
        table: "capability_belongs_to_group",
        model_table: "groups",
        belongs_to_id: GroupId,
        returns: Group,
        result: CapabilityResult,
    );
}