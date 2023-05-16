package io.matt.simple.lifeCycle;

import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.Terminated;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AllArgsConstructor;

/**
 * @author Matthew Hua
 * @data 2023/5/16
 */
public interface IllustrateWatch {

	class MasterControlProgram extends AbstractBehavior<MasterControlProgram.Command> {

		interface Command {}

		@AllArgsConstructor
		public static final class SpawnJob implements Command {
			public final String name;
		}


		public static Behavior<Command> create() {
			return Behaviors.setup(MasterControlProgram::new);
		}


		public MasterControlProgram(ActorContext<Command> context) {
			super(context);
		}

		@Override
		public Receive<Command> createReceive() {
			return newReceiveBuilder()
					.onMessage(SpawnJob.class, this::onSpawnJob)
					.onSignal(Terminated.class, this::onTerminated)
					.build();
		}


		private Behavior<Command> onSpawnJob(SpawnJob message) {
			getContext().getSystem().log().info("Spawning job {}!", message.name);
			ActorRef<Job.Command> job = getContext().spawn(Job.create(message.name), message.name);
			getContext().watch(job);
			return this;
		}


		private Behavior<Command> onTerminated(Terminated terminated) {
			getContext().getSystem().log().info("Job stopped : {}", terminated.getRef().path().name());
			return this;
		}

	}
}
