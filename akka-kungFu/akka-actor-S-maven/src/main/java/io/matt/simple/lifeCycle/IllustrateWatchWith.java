package io.matt.simple.lifeCycle;

import akka.actor.Actor;
import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AllArgsConstructor;

/**
 * @author Matthew Hua
 * @data 2023/5/16
 */
public interface IllustrateWatchWith {


	class MasterControl extends AbstractBehavior<MasterControl.Command> {

		interface Command {}

		@AllArgsConstructor
		public static final class SpawnJob implements Command {
			public final String name;
			public final ActorRef<JobDone> replyToWhenDone;
		}
		public MasterControl(ActorContext<MasterControl.Command> context) {
			super(context);
		}

		@Override
		public Receive<MasterControl.Command> createReceive() {
			return newReceiveBuilder()
					.onMessage(SpawnJob.class, this::onSpawnJob)
					.onMessage(JobTerminated.class, this::onJobTerminated)
					.build();
		}


		public static class ActorJonDone extends AbstractBehavior<JobDone> {

			public static Behavior<JobDone> create() {
				return Behaviors.setup(ActorJonDone::new);
			}

			public ActorJonDone(ActorContext<JobDone> context) {
				super(context);
			}

			@Override
			public Receive<JobDone> createReceive() {
				return null;
			}
		}

		@AllArgsConstructor
		public static final class JobDone {
			public final String name;
		}

		@AllArgsConstructor
		private static final class JobTerminated implements Command {
			final String name;
			final ActorRef<JobDone> replyToWhenDone;
		}

		public static Behavior<Command> create() {
			return Behaviors.setup(MasterControl::new);
		}

		private Behavior<Command> onSpawnJob(SpawnJob message) {
			getContext().getSystem().log().info("Spawning job {}", message.name);
			ActorRef<Job.Command> job = getContext().spawn(Job.create(message.name), message.name);
			getContext().watchWith(job, new JobTerminated(message.name, message.replyToWhenDone));
			return this;
		}

		private Behavior<Command> onJobTerminated(JobTerminated terminated) {
			getContext().getSystem().log().info("Job stopped: {}", terminated.name);
			terminated.replyToWhenDone.tell(new JobDone(terminated.name));
			return this;
		}


	}

}
