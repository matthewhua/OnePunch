package io.matt.behavior.state.game.packageMall;

public class WarehouseProcessing implements PackageState {

    private static WarehouseProcessing warehouseProcessingInstance;

    private WarehouseProcessing() {
    }

    public static WarehouseProcessing getInstance() {
        if (warehouseProcessingInstance == null) {
            warehouseProcessingInstance = new WarehouseProcessing();
        }
        return warehouseProcessingInstance;
    }

    @Override
    public void updateState(PackageContext ctx) {
        System.out.println("2 - Package is WarehouseProcessing");
        ctx.setCurrentState(InTransition.getInstance());
    }
}
