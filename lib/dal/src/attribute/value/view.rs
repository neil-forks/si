//! This module contains the [`AttributeView`] struct and its methods. This object does not exist
//! in the database.

use std::collections::{HashMap, VecDeque};

use crate::{
    AttributeReadContext, AttributeValue, AttributeValueError, AttributeValueId,
    AttributeValuePayload, AttributeValueResult, DalContext, PropKind, StandardModel,
};

/// A generated view for an [`AttributeReadContext`](crate::AttributeReadContext) and an optional
/// root [`AttributeValueId`](crate::AttributeValue). The requirements for the context are laid
/// out in [`Self::new()`].
pub struct AttributeView {
    /// The value that was generated from [`Self::new()`]. This can also be referred to as the
    /// "properties" or "tree" of the view.
    value: serde_json::Value,
}

impl AttributeView {
    /// Generates an [`AttributeView`] with an [`AttributeReadContext`](crate::AttributeReadContext)
    /// and an optional root [`AttributeValueId`](crate::AttributeValue). The context's requirements
    /// are specified in the following locations:
    ///
    /// - If the root is _not_ provided: [`AttributeValue::list_payload_for_read_context()`]
    /// - If the root is provided: [`AttributeValue::list_payload_for_read_context_and_root()`]
    ///
    /// The view is generated based on the [`AttributeValuePayloads`](crate::AttributeValuePayload)
    /// found, including their corresponding [`Props`](crate::Prop). Usually, the root should be
    /// provided if a view is desired for any given context and "location" in the object value. If
    /// the [`SchemaVariant`](crate::SchemaVariant) is known and you only desire to generate a view
    /// for the entire value, you do not need to provide the root.
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        attribute_read_context: AttributeReadContext,
        root_attribute_value_id: Option<AttributeValueId>,
    ) -> AttributeValueResult<Self> {
        let mut initial_work = match root_attribute_value_id {
            Some(root_attribute_value_id) => {
                AttributeValue::list_payload_for_read_context_and_root(
                    ctx,
                    root_attribute_value_id,
                    attribute_read_context,
                )
                .await?
            }
            None => {
                AttributeValue::list_payload_for_read_context(ctx, attribute_read_context).await?
            }
        };

        // `AttributeValueId -> serde_json pointer` so when we have a parent_attribute_value_id,
        // we know _exactly_ where in the structure we need to insert, when we have a
        // parent_attribute_resolver_id.
        let mut json_pointer_for_attribute_value_id: HashMap<AttributeValueId, String> =
            HashMap::new();

        // We sort the work queue according to the order of every nested IndexMap. This ensures that
        // when we reconstruct the final properties data, we don't have to worry about the order things
        // appear in - they are certain to be the right order.
        let attribute_value_order: Vec<AttributeValueId> = initial_work
            .iter()
            .filter_map(|avp| avp.attribute_value.index_map())
            .flat_map(|index_map| index_map.order())
            .copied()
            .collect();
        initial_work.sort_by_cached_key(|avp| {
            attribute_value_order
                .iter()
                .position(|attribute_value_id| attribute_value_id == avp.attribute_value.id())
                .unwrap_or(0)
        });

        // We need the work_queue to be a VecDeque so we can pop elements off of the front
        // as it's supposed to be a queue, not a stack.
        let mut work_queue: VecDeque<AttributeValuePayload> = VecDeque::from(initial_work);

        let mut properties = serde_json::json![{}];
        let mut root_stack: Vec<(Option<AttributeValueId>, String)> = vec![(None, "".to_string())];

        while !work_queue.is_empty() {
            let mut unprocessed: Vec<AttributeValuePayload> = vec![];
            let (root_id, json_pointer) = root_stack
                .pop()
                .ok_or(AttributeValueError::UnexpectedEmptyRootStack)?;

            while let Some(AttributeValuePayload {
                prop,
                func_binding_return_value,
                attribute_value,
                parent_attribute_value_id,
            }) = work_queue.pop_front()
            {
                if let Some(func_binding_return_value) = func_binding_return_value {
                    if let Some(found_value) = func_binding_return_value.value() {
                        if root_id == parent_attribute_value_id {
                            let insertion_pointer =
                                if let Some(parent_avi) = parent_attribute_value_id {
                                    match json_pointer_for_attribute_value_id.get(&parent_avi) {
                                        Some(ptr) => ptr.clone(),
                                        // A `None` here would mean that we're trying to process a child before we've handled its parent,
                                        // and that shouldn't be possible given how we're going through the work_queue.
                                        None => unreachable!(),
                                    }
                                } else {
                                    // After we've processed the "root" property, we shouldn't hit this case any more.
                                    json_pointer.clone()
                                };
                            let write_location = match properties.pointer_mut(&insertion_pointer) {
                                Some(write_location) => write_location,
                                None => {
                                    return Err(AttributeValueError::BadJsonPointer(
                                        insertion_pointer.clone(),
                                        properties.to_string(),
                                    ));
                                }
                            };
                            let next_json_pointer =
                                if let Some(object) = write_location.as_object_mut() {
                                    if let Some(key) = attribute_value.key() {
                                        object.insert(key.to_string(), found_value.clone());
                                        format!("{}/{}", insertion_pointer, key)
                                    } else {
                                        object.insert(prop.name().to_string(), found_value.clone());
                                        format!("{}/{}", insertion_pointer, prop.name())
                                    }
                                } else if let Some(array) = write_location.as_array_mut() {
                                    // This code can just push, because we ordered the work queue above.
                                    // Magic!
                                    array.push(found_value.clone());
                                    format!("{}/{}", insertion_pointer, array.len() - 1)
                                } else {
                                    // Note: this shouldn't ever actually get used.
                                    insertion_pointer.to_string()
                                };
                            // Record the json pointer path to *this* specific attribute resolver's location.
                            json_pointer_for_attribute_value_id
                                .insert(*attribute_value.id(), next_json_pointer.clone());

                            match prop.kind() {
                                &PropKind::Object | &PropKind::Array | &PropKind::Map => {
                                    root_stack
                                        .push((Some(*attribute_value.id()), next_json_pointer));
                                }
                                _ => {}
                            }
                        } else {
                            unprocessed.push(AttributeValuePayload::new(
                                prop,
                                Some(func_binding_return_value),
                                attribute_value,
                                parent_attribute_value_id,
                            ));
                        }
                    }
                }
            }
            work_queue = VecDeque::from(unprocessed);
        }

        Ok(Self {
            value: properties.clone(),
        })
    }

    pub fn value(&self) -> &serde_json::Value {
        &self.value
    }
}