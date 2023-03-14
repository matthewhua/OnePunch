package io.matt.behavior.iterator.generic;

public class TopicIterator implements IteratorIterator<Topic> {
    private Topic[] topics;
    private int position;

    public TopicIterator(Topic[] topics, int position) {
        this.topics = topics;
        this.position = position;
    }

    public TopicIterator(Topic[] topics) {
        this.topics = topics;
    }

    @Override
    public void reset() {
        position = 0;
    }

    @Override
    public Topic next() {
        return topics[position++];
    }

    @Override
    public Topic currentItem() {
        return topics[position];
    }

    @Override
    public boolean hasNext() {
        return position < topics.length;
    }
}
