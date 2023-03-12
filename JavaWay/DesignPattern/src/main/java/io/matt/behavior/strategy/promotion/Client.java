package io.matt.behavior.strategy.promotion;

public class Client {

    public static void main(String[] args) {
        Promotional promotional = new Promotional(new FullReduceStrategy());
        promotional.recommend("1122334455");
        Promotional promotional1 = new Promotional(new NPriceDiscountStrategy());
        promotional1.recommend("667788991010");
        Promotional promotional2 = new Promotional(new MSpikeStrategy());
        promotional2.recommend("11335577");
    }
}
