package io.matt.mapper;

import io.matt.pojo.Order;

import java.util.List;

public interface IOrderMapper {

    List<Order> findOrderAndUser();
}
