package com.matt.springwebflux.domin;


import lombok.Data;
import org.springframework.data.annotation.Id;


/**
 * 城市实体类
 *
 */
@Data
public class City {

  /**
   * 城市编号
   */
  @Id
  private Long id;

  /**
   * 省份编号
   */
  private Long provinceId;

  /**
   * 城市名称
   */
  private String cityName;

  /**
   * 描述
   */
  private String description;
}
