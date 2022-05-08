package io.matt.pojo;


import java.io.Serializable;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;

//@Table(name = "user")
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

    public List<Order> getOrderList() {
        return orderList;
    }

    public void setOrderList(List<Order> orderList) {
        this.orderList = orderList;
    }

    public List<Role> getRoleList() {
        return roleList;
    }

    public void setRoleList(List<Role> roleList) {
        this.roleList = roleList;
    }



    public Integer getId() {
        return id;
    }

    public void setId(Integer id) {
        this.id = id;
    }

    public String getUsername() {
        return username;
    }

    public void setUsername(String username) {
        this.username = username;
    }

    public String getPassword() {
        return password;
    }

    public void setPassword(String password) {
        this.password = password;
    }

    public Date getBirthday() {
        return birthday;
    }

    public void setBirthday(Date birthday) {
        this.birthday = birthday;
    }

    @Override
    public String toString() {
        return "User{" +
                "id=" + id +
                ", username='" + username + '\'' +
                ", orderList=" + orderList +
                ", roleList=" + roleList +
                '}';
    }
}
