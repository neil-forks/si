use crate::dal::test;
use dal::{BillingAccountSignup, DalContext, JwtSecretKey};

use dal::{
    test_harness::billing_account_signup, BillingAccountId, OrganizationId, ReadTenancy,
    StandardModel, WorkspaceId, WriteTenancy,
};

#[test]
async fn check_organization_specific_billing_account(
    ctx: &DalContext<'_, '_>,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let write_tenancy = WriteTenancy::new_organization(*nba.organization.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_organization_in_billing_account(
    ctx: &DalContext<'_, '_>,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy =
        ReadTenancy::new_organization(ctx.pg_txn(), vec![*nba.organization.id()], ctx.visibility())
            .await
            .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_billing_account(*nba.billing_account.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_specific_billing_account(
    ctx: &DalContext<'_, '_>,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_workspace_in_billing_account(
    ctx: &DalContext<'_, '_>,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy =
        ReadTenancy::new_workspace(ctx.pg_txn(), vec![*nba.workspace.id()], ctx.visibility())
            .await
            .expect("unable to set workspace read read_tenancy");
    assert_eq!(
        read_tenancy.billing_accounts(),
        vec![*nba.billing_account.id()]
    );
    let write_tenancy = WriteTenancy::new_billing_account(*nba.billing_account.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_specific_organization(
    ctx: &DalContext<'_, '_>,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy =
        ReadTenancy::new_organization(ctx.pg_txn(), vec![*nba.organization.id()], ctx.visibility())
            .await
            .expect("unable to set organization read read_tenancy");
    assert_eq!(
        read_tenancy.billing_accounts(),
        vec![*nba.billing_account.id()]
    );
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_workspace_in_organization(
    ctx: &DalContext<'_, '_>,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy =
        ReadTenancy::new_workspace(ctx.pg_txn(), vec![*nba.workspace.id()], ctx.visibility())
            .await
            .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(*nba.organization.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_universal(ctx: &DalContext<'_, '_>) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![BillingAccountId::from(-1)]);

    let write_tenancy = WriteTenancy::new_billing_account(1.into());
    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let write_tenancy = WriteTenancy::new_organization(1.into());
    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let write_tenancy = WriteTenancy::new_workspace(1.into());
    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let write_tenancy = WriteTenancy::new_billing_account(1.into()).into_universal();
    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);

    let write_tenancy = WriteTenancy::new_organization(1.into()).into_universal();
    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);

    let write_tenancy = WriteTenancy::new_workspace(1.into()).into_universal();
    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_billing_account_pk_identical(ctx: &DalContext<'_, '_>) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![1.into()]);
    let write_tenancy = WriteTenancy::new_billing_account(1.into());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_billing_account_pk_overlapping(ctx: &DalContext<'_, '_>) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);
    let write_tenancy = WriteTenancy::new_billing_account(2.into());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_billing_account_pk_mismatched(ctx: &DalContext<'_, '_>) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![1.into()]);
    let write_tenancy = WriteTenancy::new_billing_account(2.into());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_billing_account_pk_mismatched_level(ctx: &DalContext<'_, '_>) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![1.into()]);
    let write_tenancy = WriteTenancy::new_organization(1.into());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_organization_pk_identical(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let read_tenancy =
        ReadTenancy::new_organization(ctx.pg_txn(), vec![*nba.organization.id()], ctx.visibility())
            .await
            .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(*nba.organization.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_organization_pk_overlapping(
    ctx: &DalContext<'_, '_>,
    jwt_secret_key: &JwtSecretKey,
) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let (nba2, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let (nba3, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy = ReadTenancy::new_organization(
        ctx.pg_txn(),
        vec![
            *nba.organization.id(),
            *nba2.organization.id(),
            *nba3.organization.id(),
        ],
        ctx.visibility(),
    )
    .await
    .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(*nba2.organization.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_organization_pk_mismatched(ctx: &DalContext<'_, '_>, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy =
        ReadTenancy::new_organization(ctx.pg_txn(), vec![*nba.organization.id()], ctx.visibility())
            .await
            .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(OrganizationId::from(-1));

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_workspace_pk_identical(ctx: &DalContext<'_, '_>, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy =
        ReadTenancy::new_workspace(ctx.pg_txn(), vec![*nba.workspace.id()], ctx.visibility())
            .await
            .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_overlapping(ctx: &DalContext<'_, '_>, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let (nba2, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let (nba3, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy = ReadTenancy::new_workspace(
        ctx.pg_txn(),
        vec![
            *nba.workspace.id(),
            *nba2.workspace.id(),
            *nba3.workspace.id(),
        ],
        ctx.visibility(),
    )
    .await
    .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_workspace(*nba2.workspace.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_mismatched(ctx: &DalContext<'_, '_>, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy =
        ReadTenancy::new_workspace(ctx.pg_txn(), vec![*nba.workspace.id()], ctx.visibility())
            .await
            .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_workspace(WorkspaceId::from(-1));

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}