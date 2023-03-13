package io.matt.behavior.state.game.packageMall;

public class PackageContext {
    private PackageState currentState;
    private String packageId;

    public PackageContext(PackageState packageState, String packageId) {
        this.currentState = packageState;
        this.packageId = packageId;
        if (currentState == null) {
            this.currentState = Acknowledged.getInstance();
        }
    }

    public PackageState getCurrentState() {
        return currentState;
    }

    public void setCurrentState(PackageState currentState) {
        this.currentState = currentState;
    }

    public String getPackageId() {
        return packageId;
    }

    public void setPackageId(String packageId) {
        this.packageId = packageId;
    }

    public void update() {
        currentState.updateState(this);
    }
}
