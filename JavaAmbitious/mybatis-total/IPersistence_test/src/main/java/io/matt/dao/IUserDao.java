package io.matt.dao;

import io.matt.pojo.User;

import java.util.List;

public interface IUserDao {

    // 查询所有用户
    List<User> findAll() throws Exception;

    //根据条件进行用户查询
    User findByCondition(User user) throws Exception;
}
