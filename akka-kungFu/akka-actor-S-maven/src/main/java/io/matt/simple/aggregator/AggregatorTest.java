package io.matt.simple.aggregator;

import akka.actor.testkit.typed.javadsl.LogCapturing;
import akka.actor.testkit.typed.javadsl.TestKitJunitResource;
import akka.actor.testkit.typed.javadsl.TestProbe;
import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AllArgsConstructor;
import org.junit.ClassRule;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.JUnitCore;

import java.math.BigDecimal;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.List;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.stream.Collectors;

import static org.junit.Assert.assertEquals;

/**
 * @author Matthew Hua
 * @Date 2023/5/19
 */
public class AggregatorTest extends JUnitCore {

	@ClassRule
	public static final TestKitJunitResource testKit = new TestKitJunitResource();

	@Rule
	public final LogCapturing logCapturing = new LogCapturing();

	@Test
	public void testCollectReplies() {
		TestProbe<List<String>> testProbe = testKit.createTestProbe();
		Consumer<ActorRef<String>> sendRequests = replyTo -> {
			replyTo.tell("a");
			replyTo.tell("b");
			replyTo.tell("c");
		};
		Function<List<String>, List<String>> aggregateReplies = ArrayList::new;

		testKit.spawn(
				Aggregator.create(
						String.class,
						sendRequests,
						3,
						testProbe.getRef(),
						aggregateReplies,
						Duration.ofSeconds(3)
				)
		);

		testProbe.expectMessage(Arrays.asList("a", "b", "c"));
	}

	/**
	 * 超过就报错
	 */
	@Test
	public void testTimeOut() {
		TestProbe<List<String>> aggregateProbe = testKit.createTestProbe();
		Consumer<ActorRef<String>> sendRequests = replyTo -> {
			replyTo.tell("a");
			replyTo.tell("c");
		};
		Function<List<String>, List<String>> aggregateReplies = ArrayList::new;
		testKit.spawn(
				Aggregator.create(
						String.class,
						sendRequests,
						3,
						aggregateProbe.getRef(),
						aggregateReplies,
						Duration.ofSeconds(1)
				)
		);
		aggregateProbe.expectNoMessage(Duration.ofMillis(2000));
		aggregateProbe.expectMessage(Arrays.asList("a", "c"));

	}

	interface IllustrateUsage {

		class Hotel1 {

			@AllArgsConstructor
			public static class RequestQuote {
				public final ActorRef<Quote> replyTo;
			}

			@AllArgsConstructor
			public static class Quote {
				public final String hotel;
				public final BigDecimal price;
			}
		}

		class Hotel2 {
			@AllArgsConstructor
			public static class RequestPrice {
				public final ActorRef<Price> replyTo;
			}

			@AllArgsConstructor
			public static class Price {
				public final String hotel;
				public final BigDecimal price;
			}
		}

		class HotelCustomer extends AbstractBehavior<HotelCustomer.Command> {

			public HotelCustomer(ActorContext<Command> context, ActorRef<Hotel1.RequestQuote> hotel1, ActorRef<Hotel2.RequestPrice> hotel2) {
				super(context);

				Consumer<ActorRef<Object>> sendRequests =
						replyTo -> {
							hotel1.tell(new Hotel1.RequestQuote(replyTo.narrow()));
							hotel2.tell(new Hotel2.RequestPrice(replyTo.narrow()));
						};

				int expectedReplies = 2;
				// Object since no common type between Hotel1 and Hotel2
				context.spawnAnonymous(
						Aggregator.create(
								Object.class,
								sendRequests,
								expectedReplies,
								context.getSelf(),
								this::aggregatedQuotes,
								Duration.ofSeconds(3)));
			}

			private AggregatedQuotes aggregatedQuotes(List<Object> replies) {
				List<Quote> quotes = replies.stream()
						.map(
								r -> {
									// The hotels have different protocols with different replies,
									// convert them to `HotelCustomer.Quote` that this actor understands.
									if (r instanceof Hotel1.Quote) {
										Hotel1.Quote q = (Hotel1.Quote) r;
										return new Quote(q.hotel, q.price);
									} else if (r instanceof Hotel2.Price) {
										Hotel2.Price p = (Hotel2.Price) r;
										return new Quote(p.hotel, p.price);
									} else {
										throw new IllegalArgumentException("Unknown reply " + r);
									}
								})
						.sorted(Comparator.comparing(a -> a.price))
						.collect(Collectors.toList());
				return new AggregatedQuotes(quotes);
			}

			interface Command {
			}

			@AllArgsConstructor
			public static class AggregatedQuotes implements Command {
				public final List<Quote> quotes;
			}

			@AllArgsConstructor
			public static class Quote {
				public final String hotel;
				public final BigDecimal price;
			}

			public static Behavior<Command> create(
					ActorRef<Hotel1.RequestQuote> hotel1, ActorRef<Hotel2.RequestPrice> hotel2) {
				return Behaviors.setup(context -> new HotelCustomer(context, hotel1, hotel2));
			}

			@Override
			public Receive<Command> createReceive() {
				return newReceiveBuilder()
						.onMessage(AggregatedQuotes.class, this::onAggregatedQuotes)
						.build();
			}

			private Behavior<Command> onAggregatedQuotes(AggregatedQuotes aggregated) {
				if (aggregated.quotes.isEmpty()) getContext().getLog().info("Best Quote N/A");
				else getContext().getLog().info("Best {}", aggregated.quotes.get(0));
				return this;
			}
		}
	}
	
	@Test
	public void testUsageExample() {
		TestProbe<IllustrateUsage.Hotel1.RequestQuote> hotel1 = testKit.createTestProbe();
		TestProbe<IllustrateUsage.Hotel2.RequestPrice> hotel2 = testKit.createTestProbe();

		TestProbe<IllustrateUsage.HotelCustomer.Command> spy = testKit.createTestProbe();

		testKit.spawn(
				Behaviors.monitor(
						IllustrateUsage.HotelCustomer.Command.class,
						spy.getRef(),
						IllustrateUsage.HotelCustomer.create(hotel1.getRef(), hotel2.getRef())
				));
		hotel1.receiveMessage().replyTo.tell(new IllustrateUsage.Hotel1.Quote("#1", new BigDecimal(100)));
		hotel2.receiveMessage().replyTo.tell(new IllustrateUsage.Hotel2.Price("#2", new BigDecimal(98)));

		List<IllustrateUsage.HotelCustomer.Quote> quotes =
				spy.expectMessageClass(IllustrateUsage.HotelCustomer.AggregatedQuotes.class).quotes;
		assertEquals("#2", quotes.get(0).hotel);
		assertEquals("#1", quotes.get(1).hotel);
		assertEquals(2, quotes.size());
	}
}
