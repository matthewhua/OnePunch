package io.matt.simple.interaction;

import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.*;
import lombok.AllArgsConstructor;

import java.time.Duration;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

/**
 * @author Matthew Hua
 * @Date 2023/5/19
 */
public interface TimeTest {

	class Buncher {
		public interface Command {
		}

		public static final class Batch {
			private final List<Command> messages;

			public Batch(List<Command> messages) {
				this.messages = Collections.unmodifiableList(messages);
			}

			public List<Command> getMessages() {
				return messages;
			}

			// #timer
			@Override
			public boolean equals(Object o) {
				if (this == o) return true;
				if (o == null || getClass() != o.getClass()) return false;
				Batch batch = (Batch) o;
				return Objects.equals(messages, batch.messages);
			}

			@Override
			public int hashCode() {
				return Objects.hash(messages);
			}
		}

		@AllArgsConstructor
		public static final class ExcitingMessage implements Buncher.Command {
			public final String message;
		}

		private static final Object TIMER_KEY = new Object();

		private enum TimeOut implements Command {
			INSTANCE
		}

		public static Behavior<Command> create(ActorRef<Batch> target, Duration after, int maxSize) {
			return Behaviors.withTimers(timers -> new Buncher(timers, target, after, maxSize).idle());
		}

		private final TimerScheduler<Command> timers;
		private final ActorRef<Batch> target;
		private final Duration after;
		private final int maxSize;

		public Buncher(TimerScheduler<Command> timers, ActorRef<Batch> target, Duration after, int maxSize) {
			this.timers = timers;
			this.target = target;
			this.after = after;
			this.maxSize = maxSize;
		}

		private Behavior<Command> idle() {
			return Behaviors.receive(Command.class)
					.onMessage(Command.class, this::onIdleCommand)
					.build();
		}

		private Behavior<Command> onIdleCommand(Command command) {
			timers.startPeriodicTimer(TIMER_KEY, TimeOut.INSTANCE, after);
			return Behaviors.setup(context -> new Active(context, command));
		}

		private class Active extends AbstractBehavior<Command> {

			private final List<Command> buffer = new ArrayList<>();

			public Active(ActorContext<Command> context, Command firstCommand) {
				super(context);
				buffer.add(firstCommand);
			}

			@Override
			public Receive<Command> createReceive() {
				return newReceiveBuilder()
						.onMessage(TimeOut.class, message -> onTimeout())
						.onMessage(Command.class, this::onCommand)
						.build();
			}

			private Behavior<Command> onTimeout() {
				target.tell(new Batch(buffer));
				return idle();
			}

			private Behavior<Command> onCommand(Command message) {
				buffer.add(message);
				if (buffer.size() == maxSize) {
					timers.cancel(TIMER_KEY);
					target.tell(new Batch(buffer));
					return idle();
				} else {
					return this;
				}
			}

		}
	}

}
