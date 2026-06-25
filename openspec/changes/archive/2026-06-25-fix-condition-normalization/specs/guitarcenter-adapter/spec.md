# Delta for guitarcenter-adapter

## MODIFIED Requirements

### Requirement: Adapter SHALL extract raw condition and stickers from Algolia

The adapter MUST extract raw `condition_lvl0`/`condition_lvl1` values and derive stickers from `skuCondition`. Vocabulary mapping to `new/used/refurbished/unknown` is delegated to Rust `sanitize()`.
(Previously: adapter normalized GC conditions to 4-value vocabulary inline)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Used variants | `condition.lvl1 = "Used > Excellent"` | Extract raw condition | `condition` = `"Used > Excellent"`, no sticker |
| New | `condition.lvl0 = "New"` | Extract raw condition | `condition` = `"New"`, no sticker |
| Open Box | `skuCondition = 3` | Extract sticker | `condition` from raw value, sticker `"open_box"` |
| Blemished | `skuCondition = 11` | Extract sticker | `condition` from raw value, sticker `"blemished"` |
| Restock | `skuCondition = 2` | Extract sticker | `condition` from raw value, sticker `"restock"` |
| Unknown | No condition data | Handle absence | `condition = ""`, no sticker |
