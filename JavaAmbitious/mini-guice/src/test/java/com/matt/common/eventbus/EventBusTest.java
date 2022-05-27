package com.matt.common.eventbus;

import com.google.common.collect.Lists;
import com.google.common.eventbus.DeadEvent;
import com.google.common.eventbus.EventBus;
import com.google.common.eventbus.Subscribe;
import junit.framework.TestCase;

import java.util.ArrayList;
import java.util.List;

/**
 * @author Matthew
 * @date 2022/5/27
 */
public class EventBusTest extends TestCase {

    private static final String EVENT = "Hello";

    private static final String BUS_IDENTIFIER = "test-bus";

    private EventBus eventBus;

    @Override
    public void setUp() throws Exception {
        super.setUp();
        eventBus = new EventBus(BUS_IDENTIFIER);
    }

    public void testBasicCatcherDistribution() {
        StringCatcher catcher = new StringCatcher();
        eventBus.register(catcher);
        eventBus.post(EVENT);

        List<String> events = catcher.getEvents();
        assertEquals("Only one event should be delivered.", 1, events.size());
        assertEquals("Correct string should be delivered.", EVENT, events.get(0));
    }

    public void testPolymorphicDistribution() {
        StringCatcher catcher = new StringCatcher();
        final List<Object> objectEvents = Lists.newArrayList();
        Object objCatcher = new Object() {
            @Subscribe
            public void eat(Object food){
                objectEvents.add(food);
            }
        };
        final List<Comparable<?>> compEvents = Lists.newArrayList();
        Object compCatcher =
                new Object() {
                    @SuppressWarnings("unused")
                    @Subscribe
                    public void eat(Comparable<?> food) {
                        compEvents.add(food);
                    }
                };
        eventBus.register(catcher);
        eventBus.register(objCatcher);
        eventBus.register(compCatcher);

        //Two additional event types: Object and Comparable<?> (Played by Integer)
        Object objEvent = new Object();
        Object compEvent = 6;

        eventBus.post(EVENT);
        eventBus.post(objEvent);
        eventBus.post(compEvent);

        // Check the StringCather...
        List<String> stringEvents = catcher.getEvents();
        assertEquals("Only one String should be delivered.", 1, stringEvents.size());
        assertEquals("Correct string should be delivered.", EVENT, stringEvents.get(0));

        // Check the Catcher<Object>...
        assertEquals("Three Objects should be delivered.", 3, objectEvents.size());
        assertEquals("String fixture must be fisrt object delivered.", EVENT, objectEvents.get(0));
        assertEquals("Obejct fixture must be sccond object delivered.", compEvent, objectEvents.get(2));

        // Check the Catcher<Comparable<?>>...
        // 因为只有实现Comparable 才能加入events 里
        assertEquals("Two Comparable<?>s should be delivered.", 2, compEvents.size());
        assertEquals("String fixture must be first comparable delivered.", EVENT, compEvents.get(0));
        assertEquals(
                "Comparable fixture must be second comparable delivered.", compEvent, compEvents.get(1));
    }

    /**
     * A collector for DeadEvents.
     */
    public static class GhostCatcher {

        private List<DeadEvent> events = new ArrayList<>();

        @Subscribe
        public void ohNoesIHaveDied(DeadEvent event) {
            events.add(event);
        }

        public List<DeadEvent> getEvents() {
            return events;
        }
    }
}
