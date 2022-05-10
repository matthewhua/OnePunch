package io.matt.mapper;

import io.matt.pojo.User;
import org.apache.ibatis.annotations.Result;
import org.apache.ibatis.annotations.Results;
import org.apache.ibatis.annotations.Select;

import java.util.List;

public interface IUserMapper {

    //查询所有用户、同时查询每个用户关联的订单信息
/*    @Select("select * from user")
    @Results({
            @Result(property = "id", column = "id")
            @Result(property = "username", column = "username")
            @Result(property = "orderList", column = "id", javaType = List.class)
    })*/
    List<User> findAll();


    //查询所有用户、同时查询每个用户关联的角色信息
    List<User> findAllUserAndRole();
}
