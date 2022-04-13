import io.matt.pojo.User;
import org.apache.ibatis.io.Resources;
import org.apache.ibatis.session.SqlSession;
import org.apache.ibatis.session.SqlSessionFactory;
import org.apache.ibatis.session.SqlSessionFactoryBuilder;
import org.junit.Test;

import java.io.IOException;
import java.io.InputStream;
import java.util.List;

public class MybatisTest {

    @Test
    public void test1() throws IOException {
        // 1.Resources 工具类,配置文件的加载, 把配置文件加载成字节输入流
        InputStream resourceAsStream = Resources.getResourceAsStream("sqlMapConfig.xml");
        //2.解析了配置文件，并创建了sqlSessionFactory工厂
        SqlSessionFactory sqlSessionFactory = new SqlSessionFactoryBuilder().build(resourceAsStream);
        //3. 生产sqlSession
        SqlSession sqlSession = sqlSessionFactory.openSession(); // 默认开启一个事务,但是该事务不会自动提交
                                //在进行增删改操作时，要手动提交事务
        List<User> users = sqlSession.selectList("findAll");
        users.forEach(System.out::println);
        sqlSession.close();
    }

    @Test
    public void test2() throws IOException {
        // 1.Resources 工具类,配置文件的加载, 把配置文件加载成字节输入流
        InputStream resourceAsStream = Resources.getResourceAsStream("sqlMapConfig.xml");
        //2.解析了配置文件，并创建了sqlSessionFactory工厂
        SqlSessionFactory sqlSessionFactory = new SqlSessionFactoryBuilder().build(resourceAsStream);
        //3. 生产sqlSession
        SqlSession sqlSession = sqlSessionFactory.openSession(true);//事务自动提交
        User user = new User();
        user.setId(6);
        user.setUsername("tom");
        user.setPassword("111111");

        sqlSession.insert("saveUser", user);
        sqlSession.close();

    }

    @Test
    public void test3() throws IOException {
        InputStream resourceAsStream = Resources.getResourceAsStream("sqlMapConfig.xml");
        SqlSessionFactory sqlSessionFactory = new SqlSessionFactoryBuilder().build(resourceAsStream);
        SqlSession sqlSession = sqlSessionFactory.openSession();

        User user = new User();
        user.setId(1);
        user.setUsername("lucy");
        sqlSession.update("updateUser",user);
        sqlSession.commit();

        sqlSession.close();
    }

    @Test
    public void test4() throws IOException {
        InputStream resourceAsStream = Resources.getResourceAsStream("sqlMapConfig.xml");
        SqlSessionFactory sqlSessionFactory = new SqlSessionFactoryBuilder().build(resourceAsStream);
        SqlSession sqlSession = sqlSessionFactory.openSession();


        sqlSession.delete("io.matt.dao.IUserDao.deleteUser",12);
        sqlSession.commit();

        sqlSession.close();
    }
}
