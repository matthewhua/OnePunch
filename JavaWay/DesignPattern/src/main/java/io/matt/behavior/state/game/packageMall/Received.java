package io.matt.behavior.state.game.packageMall;

public class Received implements PackageState {

    private static Received receivedInstance;

    private Received() {
    }

    public static Received getInstance() {
        if (receivedInstance == null) {
            receivedInstance = new Received();
        }
        return receivedInstance;
    }

    @Override
    public void updateState(PackageContext ctx) {
        System.out.println("6 - Package is Received !!");
        System.out.println("=== state end ");
    }
}
