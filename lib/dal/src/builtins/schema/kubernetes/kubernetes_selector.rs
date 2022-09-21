use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::{builtins::BuiltinsResult, DalContext, Prop, PropId, PropKind, StandardModel};

use crate::builtins::schema::kubernetes::doc_url;

pub async fn create_selector_prop(
    ctx: &DalContext,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let mut selector_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "selector",
        PropKind::Object,
        Some(parent_prop_id),
        None,
    )
    .await?;
    selector_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
            )),
        )
        .await?;

    {
        let mut match_labels_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "matchLabels",
            PropKind::Map, // How to specify it as an array of strings?
            Some(*selector_prop.id()),
            None,
        )
        .await?;
        match_labels_prop
            .set_doc_link(
                ctx,
                Some(doc_url(
                    "reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
                )),
            )
            .await?;
        let mut match_labels_value_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "labelValue",
            PropKind::String,
            Some(*match_labels_prop.id()),
            None,
        )
        .await?;
        match_labels_value_prop
            .set_doc_link(
                ctx,
                Some(doc_url(
                    "reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
                )),
            )
            .await?;
    }

    Ok(selector_prop)
}