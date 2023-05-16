package io.matt.simple.interaction;

import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import akka.pattern.StatusReply;

/**
 * @author Matthew Hua
 * @Date 2023/5/16
 */
public class InteractionPatternsAskWithStatusTest {

	interface Samples {

		class Hal extends AbstractBehavior<Hal.Command> {

			public static Behavior<Hal.Command> create() {
				return Behaviors.setup(Hal::new);
			}

			public Hal(ActorContext<Command> context) {
				super(context);
			}

			public interface Command {
			}

			public static final class OpenThePodBayDoorPlease implements Hal.Command {
				public final ActorRef<StatusReply<String>> respondTo;

				public OpenThePodBayDoorPlease(ActorRef<StatusReply<String>> respondTo) {
					this.respondTo = respondTo;
				}
			}

			@Override
			public Receive<Command> createReceive() {
				return newReceiveBuilder()
						.onMessage(Hal.OpenThePodBayDoorPlease.class, this::onOpenThePodBayDoorsPlease)
						.build();
			}

			private Behavior<Hal.Command> onOpenThePodBayDoorsPlease(
					Hal.OpenThePodBayDoorPlease message) {
				message.respondTo.tell(StatusReply.error("I'm so Sorry, Dave. I'm afraid I can't do that"));
				return this;
			}
		}
	}
}
