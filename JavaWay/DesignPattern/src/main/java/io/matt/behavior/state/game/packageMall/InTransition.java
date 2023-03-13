package io.matt.behavior.state.game.packageMall;

public class InTransition implements PackageState {


    private static InTransition inTransitionInstance;

    private InTransition() {
    }

    public static InTransition getInstance() {
        if (inTransitionInstance == null) {
            inTransitionInstance = new InTransition();
        }
        return inTransitionInstance;
    }

    @Override
    public void updateState(PackageContext ctx) {
        System.out.println("3 - Package is in transition !!");
        ctx.setCurrentState(Delivering.getInstance());
    }
}
