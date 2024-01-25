package com.matt.springwebflux.controller;


import com.matt.springwebflux.domin.City;
import com.matt.springwebflux.handler.CityHandlerWithMongo;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;
import reactor.core.publisher.Flux;
import reactor.core.publisher.Mono;

@RestController
@RequestMapping("/city")
public class CityWebFluxController {

  @Autowired
  private CityHandlerWithMongo cityHandler;

  @GetMapping(value = "/{id}")
  public Mono<City> findCityById(@PathVariable("id") Long id) {
    return cityHandler.findCityById(id);
  }

  @GetMapping()
  public Flux<City> findAllCity() {
    return cityHandler.findAllCity();
  }

  @PostMapping()
  public Mono<City> saveCity(@RequestBody City city) {
    return cityHandler.save(city);
  }

  @PutMapping()
  public Mono<City> modifyCity(@RequestBody City city) {
    return cityHandler.modifyCity(city);
  }

  @DeleteMapping(value = "/{id}")
  public Mono<Long> deleteCity(@PathVariable("id") Long id) {
    return cityHandler.deleteCity(id);
  }


}


