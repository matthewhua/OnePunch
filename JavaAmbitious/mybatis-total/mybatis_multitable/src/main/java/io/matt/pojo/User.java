package io.matt.pojo;


import lombok.Data;

import java.io.Serializable;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;

//@Table(name = "user")
@Data
public class User implements Serializable {

 /*   @Id //对应的是注解id
    @GeneratedValue(strategy = GenerationType.IDENTITY) //设置主键的生成策略*/
    private Integer id;

    private String username;

    private String password;

    private Date birthday;

        //表示用户关联的订单
    private List<Order> orderList = new ArrayList<>();

    //表示用户关联的角色
    private List<Role> roleList = new ArrayList<>();

}
