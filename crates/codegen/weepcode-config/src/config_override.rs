//! Shared take/apply for `[[version_overrides]]` arrays.

use serde::de::DeserializeOwned;

use crate::deep_merge_toml;

#[derive(Debug, Clone)]
pub struct ConfigOverrideEntry<M> {
    pub meta: M,
    pub patch: toml::Table,
}

/// Strip `key` from the root table; each element is `M` + remaining keys as patch.
pub fn take_patch_array<M>(
    config: &mut toml::Value,
    key: &str,
) -> Result<Vec<ConfigOverrideEntry<M>>, toml::de::Error>
where
    M: DeserializeOwned,
{
    let Some(table) = config.as_table_mut() else {
        return Ok(Vec::new());
    };
    let Some(array_value) = table.remove(key) else {
        return Ok(Vec::new());
    };

    #[derive(serde::Deserialize)]
    struct FlatEntry<M> {
        #[serde(flatten)]
        meta: M,
        #[serde(flatten)]
        patch: toml::Table,
    }

    let entries: Vec<FlatEntry<M>> = array_value.try_into()?;
    Ok(entries
        .into_iter()
        .map(|e| ConfigOverrideEntry {
            meta: e.meta,
            patch: e.patch,
        })
        .collect())
}

/// Keys stripped from every applied patch so an override can't re-introduce a
/// nested `version_overrides` array (recursive re-injection). `campaigns` stays
/// on the list so stale patches from before the campaigns subsystem was removed
/// can't re-inject the key either. This const owns the recursive-injection keys
/// for every override kind; [`apply_patches`] takes the strip list as a
/// parameter so the strip step itself stays key-agnostic.
pub const PATCH_STRIP_KEYS: &[&str] = &["version_overrides", "campaigns"];

/// Deep-merge each patch in iteration order (later wins on a leaf), stripping
/// `strip_keys` from every patch first.
pub fn apply_patches(
    config: &mut toml::Value,
    patches: impl IntoIterator<Item = toml::Table>,
    strip_keys: &[&str],
) {
    for mut patch in patches {
        for key in strip_keys {
            patch.remove(*key);
        }
        deep_merge_toml(config, &toml::Value::Table(patch));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(s: &str) -> toml::Table {
        toml::from_str(s).unwrap()
    }

    #[test]
    fn apply_patches_strips_requested_keys() {
        let mut cfg = toml::Value::Table(table("[models]\ndefault = \"old\"\n"));
        let patch = table("[models]\ndefault = \"new\"\n");
        apply_patches(&mut cfg, std::iter::once(patch), PATCH_STRIP_KEYS);
        assert_eq!(cfg["models"]["default"].as_str(), Some("new"));

        // Top-level strip keys are removed before merge.
        let mut cfg2 = toml::Value::Table(toml::Table::new());
        let mut p = toml::Table::new();
        p.insert("version_overrides".into(), toml::Value::Array(vec![]));
        p.insert("campaigns".into(), toml::Value::Array(vec![]));
        p.insert("keep".into(), toml::Value::Boolean(true));
        apply_patches(&mut cfg2, std::iter::once(p), PATCH_STRIP_KEYS);
        assert!(cfg2.get("version_overrides").is_none());
        assert!(cfg2.get("campaigns").is_none());
        assert_eq!(cfg2["keep"].as_bool(), Some(true));
    }
}
