package com.matt.common.eventbus;

import com.google.common.collect.Lists;
import com.google.common.eventbus.Subscribe;
import junit.framework.Assert;

import javax.annotation.Nullable;
import java.util.List;

/**
 *  A simple EventSubscriber mock that records Strings.
 *
 * @author Matthew
 * @date 2022/5/27
 */
public class StringCatcher {

    private List<String> events = Lists.newArrayList();

    @Subscribe
    public void hereHaveAString(@Nullable String string){
        events.add(string);
    }

    public void methodWithoutAnnotation(@org.checkerframework.checker.nullness.qual.Nullable String string) {
        Assert.fail("Event bus must not call methods without @Subscribe!");
    }

    public List<String> getEvents() {
        return events;
    }
}
