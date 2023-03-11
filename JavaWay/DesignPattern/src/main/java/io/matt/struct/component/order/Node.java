package io.matt.struct.component.order;

import java.util.ArrayList;
import java.util.List;

public class Node extends AbstractNode {
    private List<AbstractNode> children;
    private int id;
    private int pid;

    public Node() {
        children = new ArrayList<>();
    }

    @Override
    public boolean isRoot() {
        return -1 == pid;
    }

    @Override
    public int getId() {
        return id;
    }

    @Override
    public int getParentId() {
        return pid;
    }

    @Override
    public void setId(int id) {
        this.id = id;
    }

    @Override
    public void setParentId(int parentId) {
        this.pid = parentId;
    }

    @Override
    public void add(AbstractNode c) {
        c.setParentId(this.pid + children.size());
        c.setId(c.getParentId() + 1);
        children.add(c);
        System.out.println(children);
    }

    @Override
    public void remove(AbstractNode g) {
        children.remove(g);
    }

    @Override
    public AbstractNode getChild(int i) {
        return children.get(i);
    }
}
