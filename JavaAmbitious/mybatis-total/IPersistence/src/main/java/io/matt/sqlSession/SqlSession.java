package io.matt.sqlSession;

import java.util.List;

public interface SqlSession {

    /**
     * 查询所有
     * @param statementId
     * @param params
     * @param <E>
     * @return
     * @throws Exception
     */
    public <E> List<E> selectList(String statementId, Object... params) throws Exception;

    /**
     * 根据条件查询单个
     * @param statementId
     * @param params
     * @param <T>
     * @return
     * @throws Exception
     */
    public <T> T selectOne(String statementId, Object... params) throws Exception;

    /**
     * 为Dao接口生成代理实现类
     * @param mapperClass
     * @param <T>
     * @return
     */
    public <T> T getMapper(Class<?> mapperClass);
}
