package io.matt.behavior.memento.blog;

public class Blog {
    private long id;
    private String title;
    private String content;

    public Blog(long id, String title) {
        this.id = id;
        this.title = title;
    }

    public long getId() {
        return id;
    }

    public void setId(long id) {
        this.id = id;
    }

    public String getTitle() {
        return title;
    }

    public void setTitle(String title) {
        this.title = title;
    }

    public String getContent() {
        return content;
    }

    public void setContent(String content) {
        this.content = content;
    }

    public BlogMemento createMemento() {
        BlogMemento blogMemento = new BlogMemento(id, title, content);
        return blogMemento;
    }

    public void restore(BlogMemento memento) {
        this.id = memento.getId();
        this.title = memento.getTitle();
        this.content = memento.getContent();
    }

    @Override
    public String toString() {
        return "Blog{" +
                "id=" + id +
                ", title='" + title + '\'' +
                ", content='" + content + '\'' +
                '}';
    }
}
