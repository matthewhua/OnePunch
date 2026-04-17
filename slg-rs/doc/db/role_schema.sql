-- 玩家角色表：存储基础信息
CREATE TABLE IF NOT EXISTS `p_role` (
    `role_id` BIGINT NOT NULL AUTO_INCREMENT,
    `account_id` BIGINT NOT NULL COMMENT '关联账号ID',
    `nickname` VARCHAR(64) NOT NULL DEFAULT '' COMMENT '角色名',
    `level` INT NOT NULL DEFAULT 1 COMMENT '等级',
    `serverId` INT NOT NULL DEFAULT 1 COMMENT '所属物理服ID',
    `camp` INT NOT NULL DEFAULT 1 COMMENT '阵营',
    `create_time` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `login_time` DATETIME DEFAULT NULL,
    `logout_time` DATETIME DEFAULT NULL,
    PRIMARY KEY (`role_id`),
    UNIQUE KEY `uk_nickname` (`nickname`),
    KEY `idx_account_id` (`account_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 玩家数据表：保存各系统的二进制序列化数据 (兼容 Java 模式)
-- keyId: 系统类型编号 (如 1:建筑, 2:皮肤)
CREATE TABLE IF NOT EXISTS `p_data` (
    `role_id` BIGINT NOT NULL,
    `keyId` INT NOT NULL COMMENT '系统ID',
    `data` LONGBLOB COMMENT '二进制数据',
    `update_time` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`role_id`, `keyId`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
