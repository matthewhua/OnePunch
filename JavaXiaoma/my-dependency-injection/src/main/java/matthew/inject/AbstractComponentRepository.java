package matthew.inject;

import matthew.function.ThrowableFunction;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;
import java.util.logging.Logger;
import static matthew.function.ThrowableFunction.execute;
/**
 * @author Matthew
 * @projectName Validation
 * @time 2021/10/20 11:01
 */
public abstract class AbstractComponentRepository implements ComponentRepository{

    Logger logger = Logger.getLogger(this.getClass().getName());

    LinkedHashMap<String, Object> componentsCache = new LinkedHashMap<>();

    public <C> C getComponent(String componentName){
        return (C) componentsCache.computeIfAbsent(componentName, this::doGetComponent);
    }


    public Set<String> getComponentNames(){
        return componentsCache.isEmpty() ? componentsCache.keySet() : listComponentNames();
    }


    /**
     * 通过指定 ThrowableFunction 返回计算结果
     *
     * @param argument         Function's argument
     * @param function         ThrowableFunction
     * @param ignoredException 是否忽略异常
     * @param <R>              返回结果类型
     * @return 返回
     * @see ThrowableFunction#apply(Object)
     */
    protected <T, R> R executeInContext(T argument, ThrowableFunction<T, R> function, boolean ignoredException) {
        R result = null;
        try {
            result = execute(argument, function);
        } catch (Throwable e) {
            if (ignoredException) {
                logger.warning(e.getMessage());
            } else {
                throw new RuntimeException(e);
            }
        }
        return result;
    }

    protected abstract Set<String> listComponentNames();

    protected abstract Object doGetComponent(String s);
}
