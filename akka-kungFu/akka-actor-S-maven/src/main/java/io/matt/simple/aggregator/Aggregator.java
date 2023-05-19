package io.matt.simple.aggregator;

import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AllArgsConstructor;

import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;
import java.util.function.Function;

/**
 * @author Matthew Hua
 * @Date 2023/5/19
 */
public class Aggregator<Reply, Aggregate> extends AbstractBehavior<Aggregator.CMD> {

	interface CMD {
	}

	private enum ReceiveTimeout implements CMD {
		INSTANCE
	}

	@AllArgsConstructor
	private class WrappedReply implements CMD {
		final Reply reply;
	}

	public static <R, A> Behavior<CMD> create(
			Class<R> replyClass,
			Consumer<ActorRef<R>> sendRequests,
			int expectedReplies,
			ActorRef<A> replyTo,
			Function<List<R>, A> aggregateReplies,
			Duration timeOut) {
			return Behaviors.setup(
					context ->
							new Aggregator<R, A>(
									replyClass,
									context,
									sendRequests,
									expectedReplies,
									replyTo,
									aggregateReplies,
									timeOut));
	}



	@Override
	public Receive<Aggregator.CMD> createReceive() {
		return newReceiveBuilder()
				.onMessage(WrappedReply.class, this::onReply)
				.onMessage(ReceiveTimeout.class, notUsed -> onReceiveTimeOut())
				.build();
	}

	private final int expectedReplies;
	private final ActorRef<Aggregate> replyTo;
	private final Function<List<Reply>, Aggregate> aggregateReplies;
	private final List<Reply> replies = new ArrayList<>();

	private Aggregator(
			Class<Reply> replyClass,
			ActorContext<CMD> context,
			Consumer<ActorRef<Reply>> sendRequests,
			int expectedReplies,
			ActorRef<Aggregate> replyTo,
			Function<List<Reply>, Aggregate> aggregateReplies,
			Duration timeout) {
		super(context);
		this.expectedReplies = expectedReplies;
		this.replyTo = replyTo;
		this.aggregateReplies = aggregateReplies;

		context.setReceiveTimeout(timeout, ReceiveTimeout.INSTANCE);

		ActorRef<Reply> replyAdapter = context.messageAdapter(replyClass, WrappedReply::new);
		sendRequests.accept(replyAdapter);
	}

	private Behavior<CMD> onReply(WrappedReply wrappedReply) {
		Reply reply = wrappedReply.reply;
		replies.add(reply);
		if (replies.size() == expectedReplies) {
			Aggregate result = aggregateReplies.apply(replies);
			replyTo.tell(result);
			return Behaviors.stopped();
		} else {
			return this;
		}
	}

	private Behavior<CMD> onReceiveTimeOut() {
		Aggregate result = aggregateReplies.apply(replies);
		replyTo.tell(result);
		return Behaviors.stopped();
	}


}
