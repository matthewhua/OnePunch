package io.matt.struct.component.order;

public class Leaf extends AbstractNode{
    private int id;
    private int pid;

    @Override
    public boolean isRoot() {
        return false;
    }

    @Override
    public int getId() {
        return this.id;
    }

    @Override
    public int getParentId() {
        return this.pid;
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
    public void add(AbstractNode abstractNode) {
        throw new UnsupportedOperationException("这个是叶子节点，无法增加");
    }

    @Override
    public void remove(AbstractNode g) {
        throw new UnsupportedOperationException("这个是叶子节点，无法删除");
    }

    @Override
    public AbstractNode getChild(int i) {
        return null;
    }
}
