package io.matt.struct.flyweight;


import java.util.HashMap;
import java.util.Map;

// 享元工厂类
public class FlyweightFactory {

    // 这是IDEA 自动生成的，质量蛮高
    private static final class InstanceHolder {
        private static final FlyweightFactory instance = new FlyweightFactory();
    }

    public static FlyweightFactory getInstance() {
        return InstanceHolder.instance;
    }

    // 定义一个池容器
    public Map<String, Flyweight> pool = new HashMap<>();

    public FlyweightFactory() {
        pool.put("A", new ConcreteFlyweight("A"));
        pool.put("B", new ConcreteFlyweight("B"));
        pool.put("C", new ConcreteFlyweight("C"));
    }

    // 根据内部状态来查找值
    public Flyweight getFlyweight(String key) {
        if (pool.containsKey(key)) {
            System.out.println("=== 享元池中有, 直接复用, key" + key);
            return pool.get(key);
        } else {
            if (key.length() > 1) {
                Flyweight unsharedConcreteFlyweight = new UnsharedConcreteFlyweight(key);
                return unsharedConcreteFlyweight;
            } else {
                System.out.println("===享元池中没有，重新创建并复用，key：" + key);
                Flyweight flyweightNew = new ConcreteFlyweight(key);
                pool.put(key, flyweightNew);
                return flyweightNew;
            }
        }
    }
}
