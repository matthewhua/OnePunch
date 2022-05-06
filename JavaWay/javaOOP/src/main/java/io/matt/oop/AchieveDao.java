package io.matt.oop;

/**
 * @author Matthew
 * @date 2022/5/6
 */
public class AchieveDao extends BaseDao<Integer> {


    public AchieveDao() {
        super(Integer.class);
        SimpleExecutor simpleExecutor = new SimpleExecutor();
        setX(simpleExecutor);
    }

}
