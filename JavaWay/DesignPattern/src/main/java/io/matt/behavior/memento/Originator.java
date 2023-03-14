package io.matt.behavior.memento;

public class Originator {

    private String state = "原始对象"; //打印当前状态
    private String id;
    private String name;
    private String phone;

    public Originator() {
    }

    public Originator(String id, String name, String phone) {
        this.state = state;
        this.id = id;
        this.name = name;
        this.phone = phone;
    }

    public Memento create() {
        return new Memento(id, name, phone);
    }

    public void restore(Memento memento) {
        this.state = memento.getState();
        this.id = memento.getId();
        this.phone = memento.getPhone();
        this.name = memento.getName();
    }


    @Override
    public String toString() {
        return "Originator{" +
                "state='" + state + '\'' +
                ", id='" + id + '\'' +
                ", name='" + name + '\'' +
                ", phone='" + phone + '\'' +
                '}';
    }

    public String getState() {
        return state;
    }

    public void setState(String state) {
        this.state = state;
    }

    public String getId() {
        return id;
    }

    public void setId(String id) {
        this.id = id;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getPhone() {
        return phone;
    }

    public void setPhone(String phone) {
        this.phone = phone;
    }
}
