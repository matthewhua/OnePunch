package xyz.ariane.util.memodb

import xyz.ariane.util.memodbupgrade.AbstractVersionDbDataUpgrade

/**
 * 用于实现[IEntity]中有json，blob等折叠/压缩的数据的[EntityWrapper]，
 * 这些折叠数据需要在加载到数据展开([expand])到内存数据结构中给业务逻辑使用，
 * 在执行存库操作时需要重新折叠([collapse])到[v]。
 *
 * **注意：[toEntity]不会修改[v]属性的数据，[v]中的折叠属性字段不可在除[expand]外的其他地方使用**
 *
 * Code Example:
 *
 * ```
 * @NoArgConstructor //配合wdb.save, wdb.recover使用Class.newInstance()创建实例，需要注意此实例的rankAliasMap==null，必须在expand中初始化
 * data class Alliance( // 尽可能使用data class，利用编译器自动生成/维护hashCode/equals实现
 *   public override var entity: AllianceEntity, //默认是protected的，如需直接暴露entity，可加上public扩大可见性
 *   var rankAliasMap: HashMap<Int, String> //json展开的数据结构，这里无需写默认值，统一在expand中初始化
 * ) : ExpandableEntityWrapper<AllianceEntity>() {
 *
 *   override fun fetchCheckModInterval(): Duration = Duration.ofSeconds(10L)
 *
 *   override fun copyEntity(): AllianceEntity = entity.copy()
 *
 *   /** 注意：此处只可折叠数据到参数[e]中，不可修改[entity] */
 *   override fun collapse(e: AllianceEntity) {
 *     e.rankAlias = JSON.toJSONString(rankAliasMap)
 *   }
 *
 *   override fun expand(e: AllianceEntity) {
 *     rankAliasMap = if (e.rankAlias.isNotEmpty()) {
 *       JSON.parseObject(e.rankAlias, TypeRefHashMapIntString)
 *     } else {
 *       hashMapOf() // 这里需要初始化否则rankAliasMap为null
 *     }
 *   }
 *
 *   override fun fetchPrimaryKey(): Serializable = entity.allianceId
 * }
 * ```
 */
abstract class ExpandableEntityWrapper<T : IWrapEntity> : EntityWrapper<T> {



}