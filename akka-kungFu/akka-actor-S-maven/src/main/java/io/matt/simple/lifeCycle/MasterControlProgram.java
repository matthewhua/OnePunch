package io.matt.simple.lifeCycle;

import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AllArgsConstructor;

/**
 * @author Matthew Hua
 * @data 2023/5/15
 */
public class MasterControlProgram extends AbstractBehavior<MasterControlProgram.Command> {

	interface Command {}

	@AllArgsConstructor
	public static final class SpawnJob implements Command {
		public final String name;
	}

	public enum GracefulShutdown implements Command {
		INSTANCE
	}

	public static Behavior<Command> create() {
		return  Behaviors.setup(MasterControlProgram::new);
	}

	@Override
	public Receive<Command> createReceive() {
		return newReceiveBuilder()
				.onMessage(SpawnJob.class, this::onSpawnJob)
				.onMessage(GracefulShutdown.class, message -> onGracefulShutdown())
				.build();
	}

	private Behavior<Command> onSpawnJob(SpawnJob message) {
		getContext().getSystem().log().info("Spawning job {} !", message.name);
		getContext().spawn(Job.create(message.name), message.name);
		return this;
	}

	private Behavior<Command> onGracefulShutdown() {
		getContext().getSystem().log().info("Initiating graceful shutdown...");
		return Behaviors.stopped();
	}

	public MasterControlProgram(ActorContext<Command> context) {
		super(context);
	}

	private Behavior<Command> onPostStop() {
		getContext().getSystem().log().info("Master Control Program stopped");
		return this;
	}


}
