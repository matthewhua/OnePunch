package com.matt.springwebflux.repository;

import com.matt.springwebflux.domin.City;
import org.springframework.data.mongodb.repository.ReactiveMongoRepository;
import org.springframework.stereotype.Repository;

@Repository
public interface CityRepository2 extends ReactiveMongoRepository<City, Long> {
}
