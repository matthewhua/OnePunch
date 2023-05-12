package io.matt.simple.introTest;

import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AllArgsConstructor;

public class HelloWorld extends AbstractBehavior<HelloWorld.Greet> {

	public HelloWorld(ActorContext<Greet> context) {
		super(context);
	}

	@AllArgsConstructor
	public static class Greet {
		public final String whom;
		public final ActorRef<Greeted> replyTo;
	}


	@AllArgsConstructor
	public static final class Greeted {
		public final String whom;
		public final ActorRef<Greet> from;
	}

	public static Behavior<Greet> create() {
		return Behaviors.setup(HelloWorld::new);
	}
	@Override
	public Receive<HelloWorld.Greet> createReceive() {
		return newReceiveBuilder().onMessage(Greet.class, this::onGreet).build();
	}

	private Behavior<Greet> onGreet(Greet command) {
		getContext().getLog().info("Hello {}", command.whom);
		command.replyTo.tell(new Greeted(command.whom, getContext().getSelf()));
		return this;
	}
}
