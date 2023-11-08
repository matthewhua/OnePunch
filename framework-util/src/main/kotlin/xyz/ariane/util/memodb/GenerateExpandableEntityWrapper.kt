package xyz.ariane.util.memodb

fun main(args: Array<String>) {
    // 判断参数
    val missingArgs = { "缺少参数，参数格式： [EntityWrapper类名] [Entity类名]" }
    require(args.size >= 2, missingArgs)

    val entityWrapperName = args[0].trim()
    require(entityWrapperName.isNotBlank(), missingArgs)

    val entityName = args[1].trim()
    require(entityName.isNotBlank(), missingArgs)

    val tripleQuote = "\"\"\""
    print(
        """
     @NoArgConstructor
     data class $entityWrapperName(
       public override var entity: $entityName
       // TODO 在此定义业务逻辑需要的数据结构，存储从entity中展开的数据
       // FIXME **注意：不要在这里给任何属性设置默认值，由于wdb使用编译器生成的隐藏无参构造函数实例化此对象，所有默认值都无效**
     ) : ExpandableEntityWrapper<$entityName>() {

       override fun fetchCheckModInterval(): Duration = TODO("请实现此函数")

       override fun copyEntity(): $entityName = entity.copy()

       /** 注意：此处只可折叠数据到参数[e]中，不可修改[entity] */
       override fun collapse(e: $entityName) {
         TODO($tripleQuote
           将展开的内存数据转为json等格式赋值给e中对应的字段，例如:
           e.rankAlias = JSON.toJSONString(rankAliasMap)
           $tripleQuote.trimIndent()
         )
       }

       override fun expand(e: $entityName) {
         TODO($tripleQuote
           将e中的json等数据转为对象赋值给此类的对应属性，在e中无对应数据时，需要在此处初始化非Nullable的属性，例如：
           rankAliasMap = if (e.rankAlias.isNotEmpty()) {
             JSON.parseObject(e.rankAlias, TypeRefHashMapIntString)
           } else {
             hashMapOf() // 这里需要初始化否则rankAliasMap为null
           }
           $tripleQuote.trimIndent()
         )
       }

       override fun fetchPrimaryKey(): Serializable = TODO("请实现此函数")
     }
""".trimIndent()
    )
}