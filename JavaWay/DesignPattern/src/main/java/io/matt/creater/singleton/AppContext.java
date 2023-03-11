package io.matt.creater.singleton;

import java.util.HashMap;
import java.util.Map;

/**
 * ThreadLocal 单例
 * @author: Matthew
 * @created 2022/12/26 17:43
 */
public class AppContext {
    private static final ThreadLocal<AppContext> local = new ThreadLocal<>();
    private Map<String,Object> data = new HashMap<>();

    //批量存数据
    public void setData(Map<String, Object> data) {
        getAppContext().data.putAll(data);
    }
    //存数据
    public void set(String key, String value) {
        getAppContext().data.put(key,value);
    }

    public Map<String,Object> getData() {
        return getAppContext().data;
    }

    //初始化的实现方法
    private static AppContext init(){
        final AppContext context = new AppContext();
        local.set(context);
        return context;
    }

    //做延迟初始化
    public static AppContext getAppContext() {
        AppContext context = local.get();
        if (null == context) {
            context = init();
        }
        return context;
    }

    //删除实例
    public static void remove() {
        local.remove();
    }

}
