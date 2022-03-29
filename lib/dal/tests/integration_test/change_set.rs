use dal::{
    test::{
        helpers::{create_change_set, create_edit_session, create_group},
        DalContextHeadMutRef, DalContextHeadRef,
    },
    BillingAccountId, ChangeSet, ChangeSetStatus, Group, StandardModel, Tenancy, Visibility,
    NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK,
};

use crate::dal::test;

#[test]
async fn new(DalContextHeadRef(ctx): DalContextHeadRef<'_, '_, '_>) {
    let change_set = ChangeSet::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.history_actor(),
        "mastodon rocks",
        Some(&"they are a really good band and you should like them".to_string()),
    )
    .await
    .expect("cannot create changeset");

    assert_eq!(&change_set.name, "mastodon rocks");
    assert_eq!(
        &change_set.note,
        &Some("they are a really good band and you should like them".to_string())
    );
    assert_eq!(&change_set.tenancy, &Tenancy::from(ctx.write_tenancy()));
}

#[test]
async fn apply(DalContextHeadMutRef(ctx): DalContextHeadMutRef<'_, '_, '_>, bid: BillingAccountId) {
    let mut change_set = create_change_set(ctx.txns(), ctx.history_actor(), bid).await;
    let mut edit_session = create_edit_session(ctx.txns(), ctx.history_actor(), &change_set).await;

    ctx.update_visibility(Visibility::new_edit_session(
        change_set.pk,
        edit_session.pk,
        false,
    ));

    let group = create_group(ctx).await;

    edit_session
        .save(ctx.pg_txn(), ctx.nats_txn(), ctx.history_actor())
        .await
        .expect("cannot save edit session");
    change_set
        .apply(ctx.pg_txn(), ctx.nats_txn(), ctx.history_actor())
        .await
        .expect("cannot apply change set");
    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);

    ctx.update_visibility(Visibility::new_head(false));

    let head_group = Group::get_by_id(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        group.id(),
    )
    .await
    .expect("cannot get group")
    .expect("head object should exist");

    assert_eq!(group.id(), head_group.id());
    assert_ne!(group.pk(), head_group.pk());
    assert_eq!(group.name(), head_group.name());
    assert_eq!(head_group.visibility().edit_session_pk, NO_EDIT_SESSION_PK);
    assert_eq!(head_group.visibility().change_set_pk, NO_CHANGE_SET_PK,);
}

#[test]
async fn list_open(DalContextHeadRef(ctx): DalContextHeadRef<'_, '_, '_>, bid: BillingAccountId) {
    let a_change_set = create_change_set(ctx.txns(), ctx.history_actor(), bid).await;
    let b_change_set = create_change_set(ctx.txns(), ctx.history_actor(), bid).await;
    let mut c_change_set = create_change_set(ctx.txns(), ctx.history_actor(), bid).await;

    let full_list = ChangeSet::list_open(ctx.pg_txn(), ctx.read_tenancy())
        .await
        .expect("cannot get list of open change sets");
    assert_eq!(full_list.len(), 3);
    assert!(
        full_list.iter().any(|f| f.label == a_change_set.name),
        "change set has first entry"
    );
    assert!(
        full_list.iter().any(|f| f.label == b_change_set.name),
        "change set has second entry"
    );
    assert!(
        full_list.iter().any(|f| f.label == c_change_set.name),
        "change set has third entry"
    );
    c_change_set
        .apply(ctx.pg_txn(), ctx.nats_txn(), ctx.history_actor())
        .await
        .expect("cannot apply change set");
    let partial_list = ChangeSet::list_open(ctx.pg_txn(), ctx.read_tenancy())
        .await
        .expect("cannot get list of open change sets");
    assert_eq!(partial_list.len(), 2);
    assert!(
        partial_list.iter().any(|f| f.label == a_change_set.name),
        "change set has first entry"
    );
    assert!(
        partial_list.iter().any(|f| f.label == b_change_set.name),
        "change set has second entry"
    );
}

#[test]
async fn get_by_pk(DalContextHeadRef(ctx): DalContextHeadRef<'_, '_, '_>, bid: BillingAccountId) {
    let change_set = create_change_set(ctx.txns(), ctx.history_actor(), bid).await;
    let result = ChangeSet::get_by_pk(ctx.pg_txn(), ctx.read_tenancy(), &change_set.pk)
        .await
        .expect("cannot get change set by pk")
        .expect("change set pk should exist");
    assert_eq!(&change_set, &result);
}
