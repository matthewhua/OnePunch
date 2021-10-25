package matthew.inject;

import java.util.Set;

/**
 * @author Matthew
 * @projectName Validation
 * @time 2021/10/20 10:22
 */
public interface ComponentRepository {

    /**
     * 通过名称查找组件对象
     *
     * @param name 组件名称
     * @param <C>  组件对象类型
     * @return 如果找不到返回, <code>null</code>
     */
    <C> C getComponent(String name);

    /**
     * 获取所有的组件名称
     *
     * @return
     */
    Set<String> getComponentNames();

}
