package io.matt.simple.lifeCycle;

import akka.actor.typed.Behavior;
import akka.actor.typed.PostStop;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;

/**
 * @author Matthew Hua
 * @data 2023/5/15
 */
public class Job extends AbstractBehavior<Job.Command> {

	interface Command {}

	private final String name;

	public Job(ActorContext<Command> context, String name) {
		super(context);
		this.name = name;
	}

	public static final Behavior<Command> create(String name) {
		return Behaviors.setup(context -> new Job(context, name));
	}

	@Override
	public Receive<Command> createReceive() {
		return newReceiveBuilder().onSignal(PostStop.class, postStop -> onPostStop()).build();
	}

	private Behavior<Command> onPostStop() {
		getContext().getSystem().log().info("Worker {} stopped", name);
		return this;
	}
}

