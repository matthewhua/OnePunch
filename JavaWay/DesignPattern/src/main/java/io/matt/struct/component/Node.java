package io.matt.struct.component;

import java.util.List;

public class Node extends Component{

    private List<Component> myChildren; // 存放子节点列表

    @Override
    public void operation() {
       if (myChildren != null) {
           for (Component component : myChildren) {
               component.operation();
           }
       }
    }
}
