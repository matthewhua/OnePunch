package com.gameserver.di;

import dagger.Module;

/**
 * 业务逻辑模块聚合
 * 
 * 将所有业务服务相关的模块组合在一起
 * 
 * 包含：
 * - ServiceModule：业务服务（SkillService、MapService 等）
 * 
 * 未来扩展示例：
 * - ShopModule：商城系统
 * - GuildModule：公会系统
 * - DungeonModule：副本系统
 */
@Module(includes = {
    ServiceModule.class
    // 未来可添加：ShopModule.class, GuildModule.class 等
})
public class BusinessModule {
    // 空实现，仅用于模块聚合
}
