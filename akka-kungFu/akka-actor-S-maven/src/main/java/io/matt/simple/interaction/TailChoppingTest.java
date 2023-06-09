package io.matt.simple.interaction;



import akka.actor.testkit.typed.javadsl.TestKitJunitResource;
import akka.actor.testkit.typed.javadsl.TestProbe;
import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import lombok.AllArgsConstructor;
import org.junit.ClassRule;
import org.junit.Test;

import java.time.Duration;
import java.util.function.BiFunction;

import static org.junit.Assert.assertEquals;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * @author Matthew Hua
 * @Date 2023/5/22
 */
public class TailChoppingTest {

	@ClassRule
	public static final TestKitJunitResource testKit = new TestKitJunitResource();


	@AllArgsConstructor
	static class TestReply {
		final String message;
	}

	@Test
	public void testTailChoppingOnReply() {
		TestProbe<TestReply> probe = testKit.createTestProbe(TestReply.class);
		BiFunction<Integer, ActorRef<TestReply>, Boolean> sendRequest = mock(BiFunction.class);

		when(sendRequest.apply(any(Integer.class), any(ActorRef.class))).thenReturn(true);

		Behavior<TailChopping.Command> timeout = TailChopping.create(TestReply.class, sendRequest, Duration.ofMillis(100), probe.getRef(), Duration.ofMillis(1000), new TestReply("Timeout"));

		ActorRef<TailChopping.Command> tailChoppingActor = testKit.spawn(timeout);

		//tailChoppingActor.tell(new TailChopping.WrappedReply("Success"));

		TestReply response = probe.receiveMessage();

		assertEquals("Success", response.message);
		//verify(sendRequest, times(1)).apply(any(Integer.class), any(ActorRef.class));
	}

	@Test
	public void testTailChoppingOnFinalTimeout() {
		TestProbe<TestReply> probe = testKit.createTestProbe(TestReply.class);
		BiFunction<Integer, ActorRef<TestReply>, Boolean> sendRequest = mock(BiFunction.class);

		when(sendRequest.apply(any(Integer.class), any(ActorRef.class))).thenReturn(false);

		ActorRef<TailChopping.Command> tailChoppingActor = testKit.spawn(TailChopping.create(TestReply.class, sendRequest, Duration.ofMillis(100), probe.getRef(), Duration.ofMillis(1000), new TestReply("Timeout")));

		TestReply response = probe.receiveMessage(Duration.ofMillis(1100));

		assertEquals("Timeout", response.message);
		//verify(sendRequest, times(1)).apply(any(Integer.class), any(ActorRef.class));
	}

	// Add more tests here to cover other scenarios
}