package io.matt.behavior.state.game.packageMall;

public class Delivering implements PackageState {


    private static Delivering deliveringInstance;

    private Delivering() {
    }

    public static Delivering getInstance() {
        if (deliveringInstance == null) {
            deliveringInstance = new Delivering();
        }
        return deliveringInstance;
    }

    @Override
    public void updateState(PackageContext ctx) {
        System.out.println("4 - Package is Delivering !!");
        ctx.setCurrentState(WaitForPickUp.getInstance());
    }
}
