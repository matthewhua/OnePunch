package matthew.context;

import matthew.function.ThrowableAction;
import matthew.function.ThrowableFunction;

import javax.annotation.PreDestroy;
import javax.naming.Context;
import javax.naming.InitialContext;
import javax.naming.NamingException;
import javax.servlet.ServletContext;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.logging.Logger;

/**
 * @author Matthew
 * @projectName Validation
 * @time 2021/10/20 10:28
 *
 * Java 传统组件上下文（基于 JNDI实现）
 */
public class ClassicComponentContext implements ComponentContext{


    public static final String CONTEXT_NAME = ClassicComponentContext.class.getName();

    private static final String COMPONENT_ENV_CONTEXT_NAME = "java:comp/env";

    private static final Logger logger = Logger.getLogger(CONTEXT_NAME);

    private static ServletContext servletContext; // 请注意
    // 假设一个 Tomcat JVM 进程，三个 Web Apps，会不会相互冲突？（不会冲突）
    // static 字段是 JVM 缓存吗？（是 ClassLoader 缓存）

//    private static ApplicationContext applicationContext;

//    public void setApplicationContext(ApplicationContext applicationContext){
//        ComponentContext.applicationContext = applicationContext;
//        WebApplicationContextUtils.getRootWebApplicationContext()
//    }

    private Context envContext; // Component Env Context

    private ClassLoader classLoader;

    private Map<String, Object> componentsCache = new LinkedHashMap<>();

    /**
     * @PreDestory 方法缓存, key 为标注方法, Value 为方法所属对象
     */
    private Map<Method, Object> preDestroyMethodCache = new LinkedHashMap<>();

    public static ClassicComponentContext getInstance(){
        return (ClassicComponentContext)  servletContext.getAttribute(CONTEXT_NAME);
    }

    public void init(ServletContext servletContext) throws RuntimeException{
        ClassicComponentContext.servletContext = servletContext;
        servletContext.setAttribute(CONTEXT_NAME, this);
        this.init();
    }

    @Override
    public void init() {
       initClassLoader();
       initEnvContext();
       instantiateComponents();
       initializeComponents();
       registerShutdownHook();
    }


    private void initClassLoader() {
        // 获取当前 ServletContext（WebApp）ClassLoader
        this.classLoader = servletContext.getClassLoader();
    }


    
    @Override
    public void destroy() throws RuntimeException{
        processPreDestroy();
        clearCache();
        closeEnvContext();
    }

    private void closeEnvContext() {
        close(this.envContext);
    }



    private void clearCache() {
    }


    private void initializeComponents() {

    }

    /**
     * 实例化组件
     */
    private void instantiateComponents() {
        // 遍历获取所有的组件名称
        final List<String> componentNames = listAllComponentNames();
        // 通过依赖查找, 实例化对象 (tomcat BeanFactory setter 方法的执行, 仅支持简单类型)
        componentNames.forEach(name -> componentsCache.put(name, ));
    }

    private void initEnvContext() throws RuntimeException{
        if (this.envContext != null)
            return;

        Context context = null;
        try {
            context = new InitialContext();
        } catch (NamingException e) {
            e.printStackTrace();
        }
    }



    private void registerShutdownHook() {
        Runtime.getRuntime().addShutdownHook(new Thread( () ->{
            processPreDestroy();
        }));
    }





    /**
     * @param component        组件对象
     * @param candidateMethods 候选方法
     * @see #processPreDestroy()
     */
    private void processPreDestroyMetadata(Object component, List<Method> candidateMethods)
    {
        candidateMethods.stream()
                .filter(method -> method.isAnnotationPresent(PreDestroy.class)) // 标注 @PreDestroy
                .forEach(method -> {
                    preDestroyMethodCache.put(method, component);
                });
    }




    private void processPreDestroy() {
        for (Method preDestroyMethod : preDestroyMethodCache.keySet()) {
            // 移除集合中的对象，防止重复执行 @PreDestroy 方法
            final Object remove = preDestroyMethodCache.remove(preDestroyMethod);
            // 执行目标方法
            ThrowableAction.execute(() -> preDestroyMethod.invoke(remove));
        }
    }

    /**
     * 在 Context 中执行, 通过指定 ThrowableFunction 返回计算结果
     *
     * @param function  ThrowableFunction
     * @param <R>      返回结果类型
     * @return 返回
     * @see  ThrowableFunction#apply(Object)
     */
    protected <R> R executeInContext(ThrowableFunction<Context, R> function){
        return executeInContext(function, false);
    }


    /**
     * 在 Context 中执行，通过指定 ThrowableFunction 返回计算结果
     *
     * @param function         ThrowableFunction
     * @param ignoredException 是否忽略异常
     * @param <R>              返回结果类型
     * @return 返回
     * @see ThrowableFunction#apply(Object)
     */
    protected <R> R executeInContext(ThrowableFunction<Context, R> function, boolean ignoredException) {
        return executeInContext(this.envContext, function, ignoredException);
    }

    private <R> R executeInContext(Context context, ThrowableFunction<Context,R> function, boolean ignoredException) {
        R result = null;
        try {
            result = ThrowableFunction.execute(context, function);
        } catch (Throwable e) {
           if (ignoredException)
               logger.warning(e.getMessage());
           else
               throw new RuntimeException(e);
        }
        return result;
    }


    public <C> C lookupComponent(String name){
        return executeInContext(context -> (C) context.lookup(name));
    }

    @Override
    public <C> C getComponent(String name) {
        return (C)componentsCache.get(name);
    }

    @Override
    public List<String> getComponentNames() {
        return new ArrayList<>(componentsCache.keySet());
    }


    private List<String> listAllComponentNames() {
        return listComponentNames("/");
    }

    private List<String> listComponentNames(String name) {
        executeInContext(context -> {

        })
    }


    private void close(Context envContext) {
        if (envContext != null)
            ThrowableAction.execute(envContext :: close);
    }

}
