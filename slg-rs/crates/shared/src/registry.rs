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
    /// service_name: 服务类别 (如 "home", "world")
    /// service_id: 具体实例 ID (如 "home-1")
    /// addr: 服务的访问地址 (如 "127.0.0.1:50052")
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
        
        // 1. 申请租约
        let lease_res = client.lease_grant(ttl, None).await?;
        let lease_id = lease_res.id();

        // 2. 写入 Key 并绑定租约
        client.put(key.clone(), value, Some(PutOptions::new().with_lease(lease_id))).await?;

        info!("Service registered in Etcd: {} -> {}", key, addr);

        // 3. 启动后台协程维持租约心跳
        let mut client_keep_alive = self.client.clone();
        tokio::spawn(async move {
            loop {
                match client_keep_alive.lease_keep_alive(lease_id).await {
                    Ok((mut keeper, _)) => {
                        // etcd-client 的 keep_alive 会返回一个 Stream
                        // 我们只需要触发它即可
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
        
        let resp = client.get(prefix.clone(), Some(options)).await?;
        let mut services = Vec::new();

        for kv in resp.kvs() {
            let key = kv.key_str()?.to_string();
            let val = kv.value_str()?.to_string();
            services.push((key, val));
        }

        Ok(services)
    }

    /// 发现特定服务的任意一个可用实例 (简单负载均衡)
    pub async fn discover_one(&self, service_name: &str) -> anyhow::Result<String> {
        let instances = self.discover(service_name).await?;
        instances.into_iter()
            .next()
            .map(|(_, addr)| addr)
            .ok_or_else(|| anyhow::anyhow!("No available instances for service: {}", service_name))
    }
}
