package io.matt.sqlSession;

import io.matt.pojo.Configuration;
import io.matt.pojo.MappedStatement;

import java.util.List;

public interface Executor {

    <E> List<E> query(Configuration configuration, MappedStatement mappedStatement, Object... params) throws Exception;
}
