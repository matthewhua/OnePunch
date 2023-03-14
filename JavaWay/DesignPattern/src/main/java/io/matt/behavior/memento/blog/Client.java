package io.matt.behavior.memento.blog;

public class Client {

    public static void main(String[] args) {
        Blog blog = new Blog(1, "This is my last Day");
        blog.setContent("ABC");
        System.out.println(blog);
        BlogMemento memento = blog.createMemento();
        blog.setContent("123"); //改变内容
        System.out.println(blog);
        blog.restore(memento); //撤销操作
        System.out.println(blog);
    }
}
