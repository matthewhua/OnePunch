//! Restricted GM command baseline for Step 15.
//!
//! `1113 DoSomeRq` exists in the legacy Game protocol. This module intentionally
//! supports only a tiny allow-list of safe commands so the registry has a testable
//! GM entry point without exposing destructive operations.

use anyhow::Result;
use proto::slg::{DoSomeRq, DoSomeRs};
use shared::static_config::StaticConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GmCommand {
    Ping,
    ValidateConfig,
    WhoAmI,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmContext {
    pub role_id: i64,
    pub target_role_id: Option<i64>,
}

pub struct GmSystem;

impl GmSystem {
    pub fn execute(ctx: GmContext, rq: DoSomeRq, config: &StaticConfig) -> Result<DoSomeRs> {
        let command = parse_command(&rq.str)?;
        if ctx.role_id <= 0 {
            anyhow::bail!("invalid gm role_id={}", ctx.role_id);
        }
        validate_target(&rq, ctx.target_role_id)?;

        match command {
            GmCommand::Ping => Ok(success()),
            GmCommand::ValidateConfig => {
                validate_static_config(config)?;
                Ok(success())
            }
            GmCommand::WhoAmI => Ok(success()),
        }
    }
}

fn parse_command(raw: &str) -> Result<GmCommand> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "ping" | "gm.ping" => Ok(GmCommand::Ping),
        "validate_config" | "config.validate" | "gm.validate_config" => Ok(GmCommand::ValidateConfig),
        "whoami" | "gm.whoami" => Ok(GmCommand::WhoAmI),
        "" => anyhow::bail!("empty gm command"),
        other => anyhow::bail!("unsupported gm command: {}", other),
    }
}

fn validate_target(rq: &DoSomeRq, expected_target: Option<i64>) -> Result<()> {
    let Some(raw_target) = rq.role_id.as_deref() else {
        return Ok(());
    };
    let target = raw_target
        .trim()
        .parse::<i64>()
        .map_err(|_| anyhow::anyhow!("invalid gm target role_id={}", raw_target))?;
    if target <= 0 {
        anyhow::bail!("invalid gm target role_id={}", target);
    }
    if let Some(expected) = expected_target {
        if target != expected {
            anyhow::bail!(
                "gm target mismatch: request={} expected={}",
                target,
                expected
            );
        }
    }
    Ok(())
}

fn validate_static_config(config: &StaticConfig) -> Result<()> {
    for prop in &config.shop.shop_props {
        if let Some(shop_id) = prop.shop_id {
            if !config.shop.shops.contains_key(&shop_id) {
                anyhow::bail!(
                    "invalid static config: shop_prop {} references missing shop {}",
                    prop.id,
                    shop_id
                );
            }
        }
    }
    Ok(())
}

fn success() -> DoSomeRs {
    DoSomeRs {
        success: Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::static_config::shop::{ShopConfig, StaticShopProp};
    use std::collections::HashMap;

    fn ctx() -> GmContext {
        GmContext {
            role_id: 42,
            target_role_id: Some(42),
        }
    }

    #[test]
    fn allows_only_safe_commands() {
        let rs = GmSystem::execute(
            ctx(),
            DoSomeRq {
                str: "gm.ping".to_string(),
                role_id: None,
            },
            &StaticConfig::default(),
        )
        .unwrap();
        assert_eq!(rs.success, Some(true));

        assert!(GmSystem::execute(
            ctx(),
            DoSomeRq {
                str: "add_diamond 999999".to_string(),
                role_id: None,
            },
            &StaticConfig::default(),
        )
        .is_err());
    }

    #[test]
    fn rejects_invalid_target_role() {
        assert!(GmSystem::execute(
            ctx(),
            DoSomeRq {
                str: "ping".to_string(),
                role_id: Some("99".to_string()),
            },
            &StaticConfig::default(),
        )
        .is_err());
    }

    #[test]
    fn validate_config_checks_shop_refs() {
        let mut config = StaticConfig::default();
        config.shop = ShopConfig {
            shops: HashMap::new(),
            shop_props: vec![StaticShopProp {
                id: 1,
                shop_id: Some(999),
                dsc: None,
                show_type: None,
                prop: None,
                group_val: None,
                weight: None,
                price: None,
                count: None,
                single_limit: None,
                discount: None,
                unlock_time: None,
                feature_lv: None,
                sort: None,
            }],
            props_by_shop_idx: HashMap::new(),
        };

        assert!(GmSystem::execute(
            ctx(),
            DoSomeRq {
                str: "validate_config".to_string(),
                role_id: None,
            },
            &config,
        )
        .is_err());
    }
}
