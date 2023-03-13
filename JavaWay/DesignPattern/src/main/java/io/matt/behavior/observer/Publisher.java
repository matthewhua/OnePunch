package io.matt.behavior.observer;

public interface Publisher {

    void addObserver(Observer observer);

    void removeObserver(Observer observer);

    void notify(double amt);
}
