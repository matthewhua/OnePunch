use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};

/// 地图宽度
const MAP_WIDTH: u32 = 300;
/// 地图高度
const MAP_HEIGHT: u32 = 300;
/// 每个 Sector (分区)的边长。300x300 分隔为 10x10=100 个分区。
const SECTOR_SIZE: u32 = 30;

/// 物理节点在哈希环上的虚拟节点副本数 (值越大越均衡，通常在 100-500 之间)
const VIRTUAL_NODE_REPLICAS: u32 = 256;

#[derive(Debug, Clone)]
pub struct Router {
    /// 哈希环：虚拟节点 Hash Value -> 物理节点标识 (如 NodeId, Endpoint URL)
    ring: BTreeMap<u64, String>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Self {
            ring: BTreeMap::new(),
        }
    }

    /// 将物理节点加入哈希环集群
    pub fn add_node(&mut self, node_id: &str) {
        for i in 0..VIRTUAL_NODE_REPLICAS {
            let virtual_node_key = format!("{}#{}", node_id, i);
            let hash = Self::hash_key(&virtual_node_key);
            self.ring.insert(hash, node_id.to_string());
        }
    }

    /// 从哈希环中剔除故障的物理节点
    pub fn remove_node(&mut self, node_id: &str) {
        self.ring.retain(|_, v| v != node_id);
    }

    /// 根据 Sector ID 路由到指定的物理节点
    pub fn route_by_sector(&self, sector_id: u32) -> Option<String> {
        if self.ring.is_empty() {
            return None;
        }

        let hash = Self::hash_key(&sector_id);

        // 顺时针找哈希环上第一个 `>= hash` 的虚拟节点
        match self.ring.range(hash..).next() {
            Some((_, node_id)) => Some(node_id.clone()),
            None => {
                // 走到了环的最末尾，折返回头部（首尾相接）
                self.ring.first_key_value().map(|(_, node_id)| node_id.clone())
            }
        }
    }

    /// 直接根据场景内的坐标，计算对应 Sector 再路由。超出最大值会被 clamp 到边界。
    pub fn route_by_coord(&self, x: u32, y: u32) -> Option<String> {
        let sector_id = Self::coord_to_sector(x, y);
        self.route_by_sector(sector_id)
    }

    /// 辅助方法：坐标(x,y)换算为一维的 Sector ID
    pub fn coord_to_sector(x: u32, y: u32) -> u32 {
        let max_x = x.min(MAP_WIDTH.saturating_sub(1));
        let max_y = y.min(MAP_HEIGHT.saturating_sub(1));

        let sectors_per_row = (MAP_WIDTH + SECTOR_SIZE - 1) / SECTOR_SIZE;
        let sector_x = max_x / SECTOR_SIZE;
        let sector_y = max_y / SECTOR_SIZE;

        sector_y * sectors_per_row + sector_x
    }

    /// 使用标准库自带哈希即可满足对于此低频小规模映射的性能要求
    fn hash_key<T: Hash>(item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_coord_to_sector() {
        // (0,0) -> Sector 0
        assert_eq!(Router::coord_to_sector(0, 0), 0);
        // (29, 29) -> Sector 0
        assert_eq!(Router::coord_to_sector(29, 29), 0);
        // (30, 0) -> Sector 1
        assert_eq!(Router::coord_to_sector(30, 0), 1);
        // (299, 299) -> Sector 99  (10*10 = 100个区域，最大索引99)
        assert_eq!(Router::coord_to_sector(299, 299), 99);
        // 越界值应限制在合法范围内
        assert_eq!(Router::coord_to_sector(400, 400), 99);
    }

    #[test]
    fn test_distribution_with_5_nodes() {
        let mut router = Router::new();
        // 模拟 5 个服务器节点（包含 4 个外部行省和 1 个中心圣城服的物理映射）
        let nodes = vec!["Node_Fayum", "Node_Mediterranean", "Node_Greece", "Node_CentralPlains", "Node_HolyCity"];
        
        for node in &nodes {
            router.add_node(node);
        }

        let mut distribution = HashMap::new();
        // 我们的地图共有 100 个分区
        for sector_id in 0..100 {
            let target_node = router.route_by_sector(sector_id).unwrap();
            *distribution.entry(target_node).or_insert(0) += 1;
        }

        // 打印每个节点被分配了多少个地图板块，应该相对均匀
        println!("Sector Distribution among 5 physical nodes:");
        for (node, count) in &distribution {
            println!("{}: {} sectors", node, count);
        }

        // 测试移除节点后的重新分配
        router.remove_node("Node_Fayum");
        assert_eq!(router.route_by_sector(0).is_some(), true); 
        // Node_Fayum 不应该出现在随后的路由命中中
        for sector_id in 0..100 {
            let target_node = router.route_by_sector(sector_id).unwrap();
            assert_ne!(target_node, "Node_Fayum");
        }
    }
}
