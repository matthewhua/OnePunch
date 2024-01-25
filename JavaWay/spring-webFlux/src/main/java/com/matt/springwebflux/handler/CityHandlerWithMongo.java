package com.matt.springwebflux.handler;

import com.matt.springwebflux.domin.City;
import com.matt.springwebflux.repository.CityRepository2;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import reactor.core.publisher.Flux;
import reactor.core.publisher.Mono;

@Component
public class CityHandlerWithMongo {
  private final CityRepository2 cityRepository2;

  @Autowired
  public CityHandlerWithMongo(CityRepository2 cityRepository2) {
    this.cityRepository2 = cityRepository2;
  }


  public Mono<City> save(City city) {
    return cityRepository2.save(city);
  }

  public Mono<City> findCityById(Long id) {

    return cityRepository2.findById(id);
  }

  public Flux<City> findAllCity() {

    return cityRepository2.findAll();
  }

  public Mono<City> modifyCity(City city) {

    return cityRepository2.save(city);
  }

  public Mono<Long> deleteCity(Long id) {
    cityRepository2.deleteById(id);
    return Mono.create(cityMonoSink -> cityMonoSink.success(id));
  }
}
