package io.matt.simple.interaction;

import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.*;
import lombok.AllArgsConstructor;

import java.time.Duration;
import java.util.function.BiFunction;

/**
 * @author Matthew Hua
 * @Date 2023/5/22
 */
public class TailChopping<Reply> extends AbstractBehavior<TailChopping.Command> {


	interface Command {
	}

	@Override
	public Receive<Command> createReceive() {
		return newReceiveBuilder()
				.onMessage(WrappedReply.class, this::onReply)
				.onMessage(RequestTimeout.class, notUsed -> onRequestTimeOut())
				.onMessage(FinalTimeout.class, notUsed -> onFinalTimeOut())
				.build();
	}

	public enum RequestTimeout implements Command {
		INSTANCE
	}

	public enum FinalTimeout implements Command {
		INSTANCE
	}

	@AllArgsConstructor
	public class WrappedReply implements Command {
		final Reply reply;
	}

	private final TimerScheduler<Command> timers;
	private final BiFunction<Integer, ActorRef<Reply>, Boolean> sendRequest;
	private final Duration nextRequestAfter;
	private final ActorRef<Reply> replyTo;
	private final Duration finalTimeout;
	private final Reply timeoutReply;
	private final ActorRef<Reply> replyAdapter;

	private int requestCount;

	private TailChopping(Class<Reply> replyClass, ActorContext<Command> context, TimerScheduler<Command> timers, BiFunction<Integer, ActorRef<Reply>, Boolean> sendRequest, Duration nextRequestAfter, ActorRef<Reply> replyTo, Duration finalTimeout, Reply timeoutReply) {
		super(context);
		this.timers = timers;
		this.sendRequest = sendRequest;
		this.nextRequestAfter = nextRequestAfter;
		this.replyTo = replyTo;
		this.finalTimeout = finalTimeout;
		this.timeoutReply = timeoutReply;

		replyAdapter = context.messageAdapter(replyClass, WrappedReply::new);

		sendNextRequest();
	}


	public static <R> Behavior<Command> create(Class<R> replyClass, BiFunction<Integer, ActorRef<R>, Boolean> sendRequest, Duration nextRequestAfter, ActorRef<R> replyTo, Duration finalTimeout, R timeoutReply) {
		return Behaviors.setup(context ->
				Behaviors.withTimers(timers ->
						new TailChopping<R>(
								replyClass,
								context,
								timers,
								sendRequest,
								nextRequestAfter,
								replyTo,
								finalTimeout,
								timeoutReply
						)));
	}

	private Behavior<Command> onReply(WrappedReply wrappedReply) {
		Reply reply = wrappedReply.reply;
		replyTo.tell(reply);
		return Behaviors.stopped();
	}

	private Behavior<Command> onRequestTimeOut() {
		sendNextRequest();
		return this;
	}

	private Behavior<Command> onFinalTimeOut() {
		replyTo.tell(timeoutReply);
		return Behaviors.stopped();
	}

	private void sendNextRequest() {
		requestCount++;
		if (sendRequest.apply(requestCount, replyAdapter)) {
			timers.startSingleTimer(RequestTimeout.INSTANCE, RequestTimeout.INSTANCE, nextRequestAfter);
		} else {
			timers.startPeriodicTimer(FinalTimeout.INSTANCE, FinalTimeout.INSTANCE, finalTimeout);
		}
	}

}
