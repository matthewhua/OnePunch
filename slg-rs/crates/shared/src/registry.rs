/// 服务注册与发现模块
/// 生产环境使用 etcd，开发环境可使用静态配置降级
///
/// 注意：etcd 功能需要启用 `etcd` feature，且需要系统安装 protoc。
/// 开发阶段可直接使用 AppConfig 中的静态地址，无需 etcd。

/// 简单的静态服务注册表（开发/单机模式使用）
pub struct StaticRegistry {
    auth_addr: String,
    home_addr: String,
    world_addr: String,
}

impl StaticRegistry {
    pub fn new(auth_addr: String, home_addr: String, world_addr: String) -> Self {
        Self { auth_addr, home_addr, world_addr }
    }

    pub fn auth_addr(&self) -> &str {
        &self.auth_addr
    }

    pub fn home_addr(&self) -> &str {
        &self.home_addr
    }

    pub fn world_addr(&self) -> &str {
        &self.world_addr
    }
}

/// etcd 服务注册（需要 `etcd` feature）
#[cfg(feature = "etcd")]
pub mod etcd {
    use etcd_client::{Client, PutOptions};
    use tracing::{info, error};
    use std::time::Duration;
    use tokio::time::sleep;

    pub struct EtcdRegistry {
        client: Client,
    }

    impl EtcdRegistry {
        pub async fn new(endpoints: Vec<String>) -> anyhow::Result<Self> {
            let client = Client::connect(endpoints, None).await?;
            Ok(Self { client })
        }

        /// 注册服务并保持心跳
        pub async fn register(
            &self,
            service_name: &str,
            service_id: &str,
            addr: &str,
            ttl: i64,
        ) -> anyhow::Result<()> {
            let key = format!("/services/{}/{}", service_name, service_id);
            let value = addr.to_string();
            let mut client = self.client.clone();

            let lease_res = client.lease_grant(ttl, None).await?;
            let lease_id = lease_res.id();
            client.put(key.clone(), value, Some(PutOptions::new().with_lease(lease_id))).await?;
            info!("Service registered in Etcd: {} -> {}", key, addr);

            let mut client_keep_alive = self.client.clone();
            tokio::spawn(async move {
                loop {
                    match client_keep_alive.lease_keep_alive(lease_id).await {
                        Ok((mut keeper, _)) => {
                            if let Err(e) = keeper.keep_alive().await {
                                error!("Etcd lease keep_alive error: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Etcd lease_keep_alive failed: {}", e);
                            break;
                        }
                    }
                    sleep(Duration::from_secs((ttl / 3) as u64)).await;
                }
                error!("Service registry heartbeat stopped for {}", key);
            });

            Ok(())
        }

        /// 发现特定服务的所有实例
        pub async fn discover(&self, service_name: &str) -> anyhow::Result<Vec<(String, String)>> {
            let prefix = format!("/services/{}/", service_name);
            let mut client = self.client.clone();
            let options = etcd_client::GetOptions::new().with_prefix();
            let resp = client.get(prefix, Some(options)).await?;
            let mut services = Vec::new();
            for kv in resp.kvs() {
                let key = kv.key_str()?.to_string();
                let val = kv.value_str()?.to_string();
                services.push((key, val));
            }
            Ok(services)
        }

        /// 发现特定服务的任意一个可用实例
        pub async fn discover_one(&self, service_name: &str) -> anyhow::Result<String> {
            let instances = self.discover(service_name).await?;
            instances.into_iter()
                .next()
                .map(|(_, addr)| addr)
                .ok_or_else(|| anyhow::anyhow!("No available instances for service: {}", service_name))
        }
    }
}

// 为了向后兼容，在非 etcd feature 下提供一个空的 EtcdRegistry 占位
#[cfg(not(feature = "etcd"))]
pub struct EtcdRegistry;

#[cfg(not(feature = "etcd"))]
impl EtcdRegistry {
    pub async fn new(_endpoints: Vec<String>) -> anyhow::Result<Self> {
        tracing::warn!("EtcdRegistry: etcd feature not enabled, using static fallback");
        Ok(Self)
    }

    pub async fn discover_one(&self, _service_name: &str) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("etcd feature not enabled"))
    }
}
