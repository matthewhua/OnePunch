package com.gameserver.di;

import dagger.Module;

/**
 * 基础设施模块聚合
 * 
 * 将所有基础设施相关的模块组合在一起，减少 GameServerComponent 的模块列表
 * 
 * 包含：
 * - GameServerModule：配置管理
 * - ActorSystemModule：Actor 系统
 * - DataAccessModule：数据访问层（连接池、DAO）
 */
@Module(includes = {
    GameServerModule.class,
    ActorSystemModule.class,
    DataAccessModule.class
})
public class InfrastructureModule {
    // 空实现，仅用于模块聚合
}
