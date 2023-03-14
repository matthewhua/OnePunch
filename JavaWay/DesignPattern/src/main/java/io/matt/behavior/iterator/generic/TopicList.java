package io.matt.behavior.iterator.generic;

public class TopicList implements ListList<Topic> {

    private Topic[] topics;

    public TopicList(Topic[] topics) {
        this.topics = topics;
    }

    @Override
    public IteratorIterator<Topic> iterator() {
        return new TopicIterator(topics);
    }
}
