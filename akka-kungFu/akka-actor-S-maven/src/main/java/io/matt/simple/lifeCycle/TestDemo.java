package io.matt.simple.lifeCycle;

import akka.actor.typed.ActorRef;
import akka.actor.typed.ActorSystem;
import akka.actor.typed.javadsl.Behaviors;
import org.junit.Test;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

/**
 * @author Matthew Hua
 * @data 2023/5/15
 */
public class TestDemo {

	@Test
	public void mainControl() throws InterruptedException, ExecutionException, TimeoutException {
		ActorSystem<MasterControlProgram.Command> system =
				ActorSystem.create(MasterControlProgram.create(), "SHY11");

		system.tell(new MasterControlProgram.SpawnJob("a"));
		system.tell(new MasterControlProgram.SpawnJob("b"));

		// sleep here to allow time for the new actors to be started
		Thread.sleep(100);
		system.tell(MasterControlProgram.GracefulShutdown.INSTANCE);
		system.getWhenTerminated().toCompletableFuture().get(3, TimeUnit.SECONDS);
	}


	@Test
	public void IllustrateWatch() throws InterruptedException, ExecutionException, TimeoutException {
		ActorSystem<IllustrateWatch.MasterControlProgram.Command> system =
				ActorSystem.create(IllustrateWatch.MasterControlProgram.create(), "SHY11");

		system.tell(new IllustrateWatch.MasterControlProgram.SpawnJob("a"));
		system.tell(new IllustrateWatch.MasterControlProgram.SpawnJob("b"));

		// sleep here to allow time for the new actors to be started
		Thread.sleep(100);
		system.terminate();
		system.getWhenTerminated().toCompletableFuture().get(3, TimeUnit.SECONDS);
	}


	@Test
	public void IllustrateWatchWith() throws InterruptedException, ExecutionException, TimeoutException {
		ActorSystem<IllustrateWatchWith.MasterControl.Command> system =
				ActorSystem.create(IllustrateWatchWith.MasterControl.create(), "SHY12");
		Behaviors.setup(context -> {

			ActorRef<IllustrateWatchWith.MasterControl.JobDone> jobDone = context.spawn(IllustrateWatchWith.MasterControl.ActorJonDone.create(), "JobDone");
			system.tell(new IllustrateWatchWith.MasterControl.SpawnJob("a", jobDone));
			system.tell(new IllustrateWatchWith.MasterControl.SpawnJob("b", jobDone));

			return Behaviors.stopped();
		});


		// sleep here to allow time for the new actors to be started
		Thread.sleep(100);
		system.getWhenTerminated().toCompletableFuture().get(3, TimeUnit.SECONDS);
	}
}
