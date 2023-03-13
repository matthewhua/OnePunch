package io.matt.behavior.state.game.packageMall;

public class Acknowledged implements PackageState {

    private static Acknowledged acknowledgedInstance;

    private Acknowledged() {
    }

    public static Acknowledged getInstance() {
        if (acknowledgedInstance == null) {
            acknowledgedInstance = new Acknowledged();
        }
        return acknowledgedInstance;
    }


    @Override
    public void updateState(PackageContext ctx) {
        System.out.println("==== state start...");
        System.out.println("1 - Package is acknowledged !!");
        ctx.setCurrentState(WarehouseProcessing.getInstance());
    }


}
