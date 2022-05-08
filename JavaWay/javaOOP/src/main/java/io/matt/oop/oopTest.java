package io.matt.oop;

/**
 * @author Matthew
 * @date 2022/5/6
 */
public class oopTest {

    public static void main(String[] args) {
        AchieveDao dao = new AchieveDao();
        Class entityClazz = dao.getEntityClazz();
        System.out.println(entityClazz.getName());
        ActivityCatDao activityCatDao = new ActivityCatDao();
        Class catDaoEntityClazz = activityCatDao.getEntityClazz();
        System.out.println(catDaoEntityClazz);
    }
}
