//! Shop system baseline.
//!
//! Owns per-player shop state (`p_data.shop_func`) and provides pure planning
//! helpers. Cross-system effects (resource/item payment and rewards) are
//! orchestrated by the Home registry so the system itself stays isolated.

use anyhow::Result;
use prost::Message;
use proto::slg::{AwardPb, ShopDataFunction, ShopInfo, ShopItemInfo};
use shared::persistence::col;
use shared::static_config::shop::{ShopConfig, StaticShopProp};
use tracing::info;

use super::PlayerSystem;

#[derive(Debug, Clone)]
pub struct ShopSystem {
    dirty: bool,
    data: ShopDataFunction,
}

#[derive(Debug, Clone)]
pub struct ShopPurchasePlan {
    pub shop_id: i32,
    pub slot: i32,
    pub buy_count: i32,
    pub item_cfg_id: i32,
    pub prices: Vec<AwardPb>,
    pub rewards: Vec<AwardPb>,
}

impl ShopSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            data: ShopDataFunction::default(),
        }
    }

    pub fn to_proto(&self) -> ShopDataFunction {
        self.data.clone()
    }

    pub fn ensure_configured(&mut self, config: &ShopConfig) {
        let next = build_shop_data(config, &self.data);
        self.data = next;
    }

    pub fn plan_buy(
        &self,
        shop_id: i32,
        slot: i32,
        buy_count: i32,
        config: &ShopConfig,
    ) -> Result<ShopPurchasePlan> {
        if buy_count <= 0 {
            anyhow::bail!("invalid shop buy count: {}", buy_count);
        }

        let item = self.find_item(shop_id, slot).ok_or_else(|| {
            anyhow::anyhow!("shop item not found: shop_id={} slot={}", shop_id, slot)
        })?;
        let item_cfg_id = item.item_cfg_id.ok_or_else(|| {
            anyhow::anyhow!(
                "shop item missing item_cfg_id: shop_id={} slot={}",
                shop_id,
                slot
            )
        })?;
        let prop = config
            .shop_props
            .iter()
            .find(|prop| prop.id == item_cfg_id)
            .ok_or_else(|| anyhow::anyhow!("shop prop config not found: {}", item_cfg_id))?;

        let purchased = item.purchased_count.unwrap_or_default();
        if let Some(limit) = prop.single_limit.filter(|limit| *limit > 0) {
            if purchased.saturating_add(buy_count) > limit {
                anyhow::bail!(
                    "shop item purchase limit exceeded: item_cfg_id={} purchased={} buy_count={} limit={}",
                    item_cfg_id,
                    purchased,
                    buy_count,
                    limit
                );
            }
        }

        let prices = parse_award_list(prop.price.as_deref().unwrap_or_default(), buy_count)?;
        let rewards = parse_award_list(prop.prop.as_deref().unwrap_or_default(), buy_count)?;
        if rewards.is_empty() {
            anyhow::bail!("shop prop {} has no rewards", item_cfg_id);
        }

        Ok(ShopPurchasePlan {
            shop_id,
            slot,
            buy_count,
            item_cfg_id,
            prices,
            rewards,
        })
    }

    pub fn apply_buy(&mut self, plan: &ShopPurchasePlan) -> Result<ShopItemInfo> {
        let item = self.find_item_mut(plan.shop_id, plan.slot).ok_or_else(|| {
            anyhow::anyhow!(
                "shop item not found when applying buy: shop_id={} slot={}",
                plan.shop_id,
                plan.slot
            )
        })?;
        item.purchased_count = Some(item.purchased_count.unwrap_or_default() + plan.buy_count);
        item.purchased_count_ex =
            Some(item.purchased_count_ex.unwrap_or_default() + plan.buy_count);
        let result = item.clone();
        self.dirty = true;
        Ok(result)
    }

    pub fn find_item(&self, shop_id: i32, slot: i32) -> Option<&ShopItemInfo> {
        self.data
            .shop
            .iter()
            .find(|shop| shop.shop_id == Some(shop_id))?
            .item
            .iter()
            .find(|item| item.slot == Some(slot))
    }

    fn find_item_mut(&mut self, shop_id: i32, slot: i32) -> Option<&mut ShopItemInfo> {
        self.data
            .shop
            .iter_mut()
            .find(|shop| shop.shop_id == Some(shop_id))?
            .item
            .iter_mut()
            .find(|item| item.slot == Some(slot))
    }
}

impl Default for ShopSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerSystem for ShopSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        self.data = ShopDataFunction::decode(data)?;
        info!(shops = self.data.shop.len(), "ShopSystem loaded");
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.data.encode_to_vec())
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn column_name(&self) -> &'static str {
        col::SHOP
    }

    fn handle_command(
        &mut self,
        cmd: u32,
        _payload: &[u8],
        _config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("Unsupported shop cmd: {}", cmd))
    }
}

impl shared::msg::ToFunctionClientBaseBytes for ShopSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(func_type::SHOP, func_tag::SHOP, &self.data)
    }
}

pub fn parse_award_list(input: &str, multiplier: i32) -> Result<Vec<AwardPb>> {
    if multiplier <= 0 {
        anyhow::bail!("invalid award multiplier: {}", multiplier);
    }

    let input = input.trim();
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let rows = if let Ok(single) = serde_json::from_str::<Vec<i64>>(input) {
        vec![single]
    } else if let Ok(rows) = serde_json::from_str::<Vec<Vec<i64>>>(input) {
        rows
    } else {
        input
            .split(';')
            .filter(|seg| !seg.trim().is_empty())
            .map(|seg| {
                seg.split(',')
                    .filter_map(|part| part.trim().parse::<i64>().ok())
                    .collect::<Vec<_>>()
            })
            .collect()
    };

    rows.into_iter()
        .map(|parts| award_from_parts(&parts, i64::from(multiplier)))
        .collect()
}

fn award_from_parts(parts: &[i64], multiplier: i64) -> Result<AwardPb> {
    if parts.len() < 3 {
        anyhow::bail!("invalid award parts: {:?}", parts);
    }
    let count = parts[2]
        .checked_mul(multiplier)
        .ok_or_else(|| anyhow::anyhow!("award count overflow: {:?} x {}", parts, multiplier))?;
    Ok(AwardPb {
        r#type: parts[0] as i32,
        id: parts[1] as i32,
        count,
        safe: Some(true),
        ..Default::default()
    })
}

fn build_shop_data(config: &ShopConfig, existing: &ShopDataFunction) -> ShopDataFunction {
    let mut shops: Vec<ShopInfo> = config
        .shops
        .keys()
        .filter_map(|shop_id| build_shop_info(*shop_id, config, existing))
        .collect();
    shops.sort_by_key(|shop| shop.shop_id.unwrap_or_default());
    ShopDataFunction { shop: shops }
}

fn build_shop_info(
    shop_id: i32,
    config: &ShopConfig,
    existing: &ShopDataFunction,
) -> Option<ShopInfo> {
    let shop_config = config.shops.get(&shop_id)?;
    // Step 15 baseline supports fixed shops only. Wander/mysterious shops need
    // weighted slot generation and refresh lifecycle, so avoid exposing every
    // pool item as directly buyable until that behavior is implemented.
    if shop_config.shop_type != Some(3) {
        return None;
    }

    let existing_shop = existing
        .shop
        .iter()
        .find(|shop| shop.shop_id == Some(shop_id));
    let mut prop_indexes = config
        .props_by_shop_idx
        .get(&shop_id)
        .cloned()
        .unwrap_or_default();
    prop_indexes.sort_by_key(|idx| {
        let prop = &config.shop_props[*idx];
        (prop.sort.unwrap_or(i32::MAX), prop.id)
    });

    let mut items = Vec::new();
    for (idx, prop_index) in prop_indexes.into_iter().enumerate() {
        let prop = &config.shop_props[prop_index];
        let slot = (idx + 1) as i32;
        let previous = existing_shop.and_then(|shop| {
            shop.item
                .iter()
                .find(|item| item.item_cfg_id == Some(prop.id))
        });
        items.push(shop_item_from_config(slot, prop, previous));
    }

    Some(ShopInfo {
        shop_id: Some(shop_id),
        shop_end_time: existing_shop.and_then(|shop| shop.shop_end_time),
        refresh_time: existing_shop.and_then(|shop| shop.refresh_time),
        refreshed_count: existing_shop.and_then(|shop| shop.refreshed_count),
        item: items,
    })
}

fn shop_item_from_config(
    slot: i32,
    prop: &StaticShopProp,
    previous: Option<&ShopItemInfo>,
) -> ShopItemInfo {
    ShopItemInfo {
        slot: Some(slot),
        item_cfg_id: Some(prop.id),
        purchased_count: previous.and_then(|item| item.purchased_count).or(Some(0)),
        purchased_count_ex: previous
            .and_then(|item| item.purchased_count_ex)
            .or(Some(0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn shop_prop(id: i32, shop_id: i32, sort: i32, price: &str, prop: &str) -> StaticShopProp {
        StaticShopProp {
            id,
            shop_id: Some(shop_id),
            dsc: None,
            show_type: None,
            prop: Some(prop.to_string()),
            group_val: None,
            weight: None,
            price: Some(price.to_string()),
            count: None,
            single_limit: Some(2),
            discount: None,
            unlock_time: None,
            feature_lv: None,
            sort: Some(sort),
        }
    }

    fn shop_config() -> ShopConfig {
        let mut shops = HashMap::new();
        shops.insert(
            1,
            shared::static_config::shop::StaticShop {
                id: 1,
                name: Some("test".to_string()),
                shop_type: Some(3),
                show_type: None,
                show_res: None,
                limited_time: None,
                manual_refresh: None,
                free_refresh: None,
                refresh_need: None,
                slot_num: Some(2),
                refresh_type: None,
                sort: Some(1),
                field_configuration: None,
                group_str: None,
                function_open: None,
                form_id: None,
            },
        );
        let shop_props = vec![
            shop_prop(1002, 1, 2, "[1,3,50]", "[4,2002,1]"),
            shop_prop(1001, 1, 1, "[1,3,20]", "[4,2001,2]"),
        ];
        let props_by_shop_idx = HashMap::from([(1, vec![0, 1])]);
        ShopConfig {
            shops,
            shop_props,
            props_by_shop_idx,
        }
    }

    #[test]
    fn parses_single_and_multiple_awards() {
        let single = parse_award_list("[1,3,50]", 2).unwrap();
        assert_eq!(single.len(), 1);
        assert_eq!(single[0].r#type, 1);
        assert_eq!(single[0].id, 3);
        assert_eq!(single[0].count, 100);

        let multiple = parse_award_list("[[4,1001,2],[1,2,5]]", 1).unwrap();
        assert_eq!(multiple.len(), 2);
        assert_eq!(multiple[0].id, 1001);
        assert_eq!(multiple[1].r#type, 1);
    }

    #[test]
    fn configured_shop_uses_sort_order_and_preserves_purchased_count() {
        let config = shop_config();
        let mut system = ShopSystem::new();
        system.ensure_configured(&config);
        assert_eq!(system.to_proto().shop[0].item[0].item_cfg_id, Some(1001));
        assert_eq!(system.to_proto().shop[0].item[1].item_cfg_id, Some(1002));

        let plan = system.plan_buy(1, 1, 1, &config).unwrap();
        let item = system.apply_buy(&plan).unwrap();
        assert_eq!(item.purchased_count, Some(1));

        system.ensure_configured(&config);
        assert_eq!(system.find_item(1, 1).unwrap().purchased_count, Some(1));
    }

    #[test]
    fn plan_buy_enforces_single_limit() {
        let config = shop_config();
        let mut system = ShopSystem::new();
        system.ensure_configured(&config);
        let first = system.plan_buy(1, 1, 2, &config).unwrap();
        system.apply_buy(&first).unwrap();

        let err = system.plan_buy(1, 1, 1, &config).unwrap_err();
        assert!(err.to_string().contains("purchase limit"));
    }
}
