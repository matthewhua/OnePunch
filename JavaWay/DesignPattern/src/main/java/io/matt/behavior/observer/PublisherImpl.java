package io.matt.behavior.observer;

import java.util.ArrayList;
import java.util.List;

public class PublisherImpl implements Publisher {

    private String acct;
    private double balance;
    private List<Observer> myObservers;

    public PublisherImpl(String acct, double balance) {
        this.acct = acct;
        this.balance = balance;
        this.myObservers = new ArrayList<>();
    }

    @Override
    public void addObserver(Observer observer) {
        myObservers.add(observer);
    }

    @Override
    public void removeObserver(Observer observer) {
        myObservers.remove(observer);
    }

    @Override
    public void notify(double amt) {
        this.balance -= amt;
        if (balance < 0) {
            overdrawn();
        }
    }

    private void overdrawn() {
        for (Observer myObserver : myObservers) {
            myObserver.notify(acct, balance);
        }
    }

}
