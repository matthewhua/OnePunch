package io.matt.oop;

/**
 * @author Matthew
 * @date 2022/5/6
 */
public abstract class BaseDao<T> {

    Executor x;

    Class<?> obj;

    T t;

    public void add(T t){
        this.t = t;
    }

    public T get(){
        return t;
    }

    public Class<?> getObj(){
        return t.getClass();
    }


    public BaseDao(Class obj) {
        this.obj = obj;
    }

    public void setX(Executor x) {
        this.x = x;
    }

    protected  Class getEntityClazz(){
       if (x != null) {
           x.execute(() -> {});
       }
        return obj;
    }
}
