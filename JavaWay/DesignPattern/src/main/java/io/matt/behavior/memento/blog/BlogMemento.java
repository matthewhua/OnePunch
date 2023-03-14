package io.matt.behavior.memento.blog;

public class BlogMemento {

    private final long id;
    private final String title;
    private final String content;

    public BlogMemento(long id, String title, String content) {
        super();
        this.id = id;
        this.title = title;
        this.content = content;
    }

    public long getId() {
        return id;
    }

    public String getTitle() {
        return title;
    }

    public String getContent() {
        return content;
    }
}
