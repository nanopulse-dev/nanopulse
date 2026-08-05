#![allow(clippy::module_inception)]

mod profile;
mod vendor;

pub use profile::*;
pub use vendor::*;

pub fn json_diff(old: &serde_json::Value, new: &serde_json::Value) -> serde_json::Value {
    match (old, new) {
        (serde_json::Value::Object(old_obj), serde_json::Value::Object(new_obj)) => {
            let mut out = serde_json::Map::new();

            for (k, v) in new_obj {
                match old_obj.get(k) {
                    Some(old_v) => {
                        if v != old_v {
                            out.insert(k.into(), v.clone());
                        }
                    }
                    None => {
                        out.insert(k.into(), v.clone());
                    }
                }
            }
            if out.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::Object(out)
            }
        }
        _ => {
            if old == new {
                serde_json::Value::Null
            } else {
                new.clone()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_diff() {
        let a = json!({"foo": "bar", "test": "bar"});
        let b = json!({"foo": "bar", "test": "foo"});
        let expected = json!({"test": "foo"});

        assert_eq!(expected, json_diff(&a, &b));
    }
}
