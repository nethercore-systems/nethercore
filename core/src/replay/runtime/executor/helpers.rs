//! Helper functions for executor

use crate::replay::script::CompiledAssertValue;
use crate::replay::types::DebugValueData;
use hashbrown::HashMap;
use std::collections::BTreeMap;

/// Compute the delta between pre and post values
pub(super) fn compute_delta(
    pre: &HashMap<String, DebugValueData>,
    post: &HashMap<String, DebugValueData>,
) -> Option<BTreeMap<String, String>> {
    let mut delta = BTreeMap::new();

    for (name, post_value) in post {
        if let Some(pre_value) = pre.get(name) {
            let changed = match (pre_value, post_value) {
                (DebugValueData::I32(a), DebugValueData::I32(b)) if a != b => {
                    Some(format!("{}", b - a))
                }
                (DebugValueData::U32(a), DebugValueData::U32(b)) if a != b => {
                    Some(format!("{}", (*b as i64) - (*a as i64)))
                }
                (DebugValueData::F32(a), DebugValueData::F32(b))
                    if (a - b).abs() > f32::EPSILON =>
                {
                    Some(format!("{:.2}", b - a))
                }
                (DebugValueData::F64(a), DebugValueData::F64(b))
                    if (a - b).abs() > f64::EPSILON =>
                {
                    Some(format!("{:.2}", b - a))
                }
                (DebugValueData::Bool(a), DebugValueData::Bool(b)) if a != b => {
                    Some(format!("{} -> {}", a, b))
                }
                _ => None,
            };

            if let Some(change) = changed {
                delta.insert(name.clone(), change);
            }
        }
    }

    if delta.is_empty() { None } else { Some(delta) }
}

pub(super) fn map_to_btree(
    values: HashMap<String, DebugValueData>,
) -> BTreeMap<String, DebugValueData> {
    values.into_iter().collect()
}

/// Format the expected value for error messages
pub(super) fn format_expected(value: &CompiledAssertValue, resolved: Option<f64>) -> String {
    match value {
        CompiledAssertValue::Number(n) => format!("{}", n),
        CompiledAssertValue::Variable(name) => {
            if let Some(v) = resolved {
                format!("{} ({})", name, v)
            } else {
                format!("{} (undefined)", name)
            }
        }
        CompiledAssertValue::PrevValue(name) => {
            if let Some(v) = resolved {
                format!("$prev_{} ({})", name, v)
            } else {
                format!("$prev_{} (undefined)", name)
            }
        }
    }
}
