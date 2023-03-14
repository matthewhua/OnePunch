package io.matt.behavior.memento;

public class Demo {

    public static void main(String[] args) {
        Originator originator = new Originator();
        originator.setId("1");
        originator.setName("Matthew");
        originator.setPhone("17601353709");
        System.out.println(originator);
        Memento memento = originator.create();
        memento.setName("修改");
        System.out.println(originator);
        originator.restore(memento);
        System.out.println(originator);
    }
}
