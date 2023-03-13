package io.matt.behavior.state.game.packageMall;

public class WaitForPickUp implements PackageState {


    private static WaitForPickUp waitForPickUpInstance;

    private WaitForPickUp() {
    }

    public static WaitForPickUp getInstance() {
        if (waitForPickUpInstance == null) {
            waitForPickUpInstance = new WaitForPickUp();
        }
        return waitForPickUpInstance;
    }

    @Override
    public void updateState(PackageContext ctx) {
        System.out.println("5 - Package is waiting for pick up !!");
        ctx.setCurrentState(Received.getInstance());
    }
}
