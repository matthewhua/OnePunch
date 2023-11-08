package xyz.ariane.util.zk

import xyz.ariane.util.annotation.AllOpen
import xyz.ariane.util.concurrent.LockObj

/**
 * 模板管理
 */
@AllOpen
class ZkTemplateMgr<T, V>(val fetchId: (T) -> V) {
    private val templateMap = hashMapOf<V, T>()

    private val lockObj = LockObj()

    fun addOrUpdateTemplate(template: T) {
        synchronized(lockObj) {
            templateMap[fetchId(template)] = template
        }
    }

    fun addOrUpdateTemplates(templates: List<T>) {
        synchronized(lockObj) {
            for (template in templates) {
                templateMap[fetchId(template)] = template
            }
        }
    }

    fun removeTemplate(template: T) {
        synchronized(lockObj) {
            templateMap.remove(fetchId(template))
        }
    }

    fun fetchTemplates(): Map<V, T> {
        synchronized(lockObj) {
            return templateMap.toMap()
        }
    }

    fun fetchTemplateList(): List<T> {
        synchronized(lockObj) {
            return templateMap.values.toList()
        }
    }

    fun fetchTemplateById(id: V): T? {
        synchronized(lockObj) {
            return templateMap[id]
        }
    }
}