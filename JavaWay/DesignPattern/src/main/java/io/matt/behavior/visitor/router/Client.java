package io.matt.behavior.visitor.router;

import java.util.ArrayList;
import java.util.List;

public class Client {

    public static void main(String[] args) {
        RouterVisitor linuxRouterVisitor = new LinuxRouterVisitor();
        RouterVisitor windowsRouterVisitor = new WindowsRouterVisitor();
        List<Router> routers = new ArrayList<>();
        routers.add(new DLinkRouter());
        routers.add(new ASAURouter());
        routers.add(new TPLinkRouter());
        for (Router router : routers) {
            router.accept(linuxRouterVisitor);
            router.accept(windowsRouterVisitor);
        }
    }
}
