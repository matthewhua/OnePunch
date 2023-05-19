package io.matt.simple.interaction;

import akka.Done;
import akka.actor.testkit.typed.javadsl.LogCapturing;
import akka.actor.testkit.typed.javadsl.TestKitJunitResource;
import akka.actor.testkit.typed.javadsl.TestProbe;
import akka.actor.typed.ActorRef;
import akka.actor.typed.ActorSystem;
import org.junit.ClassRule;
import org.junit.Rule;
import org.junit.Test;

import java.time.Duration;
import java.util.Arrays;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.Assert.assertEquals;

/**
 * @author Matthew Hua
 * @Date 2023/5/16
 */
public class TestMain {

	@ClassRule
	public static final TestKitJunitResource testKit = new TestKitJunitResource();

	@Rule
	public final LogCapturing logCapturing = new LogCapturing();

	@Test
	public void fireAndForgetSample() throws ExecutionException, InterruptedException, TimeoutException {
		ActorSystem<FireAndForget.Printer.PrintMe> system =
				ActorSystem.create(FireAndForget.Printer.create(), "printer-sample-system");

		//note that system is also the ActorRef to the guardian actor
		final ActorSystem<FireAndForget.Printer.PrintMe> ref = system;

		// these are all fire and forget
		ref.tell(new FireAndForget.Printer.PrintMe("message1"));
		ref.tell(new FireAndForget.Printer.PrintMe("message2"));
		// #fire-and-forget-doit

		system.terminate();
		system.getWhenTerminated().toCompletableFuture().get(5, TimeUnit.SECONDS);
	}

	@Test
	public void askWithStatusExample() {

// no assert but should at least throw if completely broken
		ActorRef<StandaloneAskSample.CookieFabric.CommandOne> cookieFabric =
				testKit.spawn(StandaloneAskSample.CookieFabric.createOne());
		NotShown notShown = new NotShown();
		notShown.askAndPrint(testKit.system(), cookieFabric);

	}

	@Test
	public void testPipeToSelf() {
		PipeToSelfSample.CustomerDataAccess dataAccess =
				customer -> CompletableFuture.completedFuture(Done.getInstance());

		ActorRef<PipeToSelfSample.CustomerRepository.Command> repository =
				testKit.spawn(PipeToSelfSample.CustomerRepository.create(dataAccess));
		TestProbe<PipeToSelfSample.CustomerRepository.OperationResult> probe =
				testKit.createTestProbe(PipeToSelfSample.CustomerRepository.OperationResult.class);

		repository.tell(
				new PipeToSelfSample.CustomerRepository.Update(
						new PipeToSelfSample.Customer("123", 1L, "Alice", "Fairy tail road 7"),
						probe.getRef()));

		assertEquals(
				"123",
				probe.expectMessageClass(PipeToSelfSample.CustomerRepository.UpdateSuccess.class).id);

	}

	@Test
	public void timers() throws Exception {
		TestProbe<TimeTest.Buncher.Batch> probe = testKit.createTestProbe(TimeTest.Buncher.Batch.class);
		ActorRef<TimeTest.Buncher.Command> buncher = testKit.spawn(TimeTest.Buncher.create(probe.ref(), Duration.ofSeconds(1), 10), "batcher");

		TimeTest.Buncher.ExcitingMessage one = new TimeTest.Buncher.ExcitingMessage("one");
		TimeTest.Buncher.ExcitingMessage two = new TimeTest.Buncher.ExcitingMessage("two");

		buncher.tell(one);
		buncher.tell(two);

		probe.expectNoMessage();
		probe.expectMessage(Duration.ofSeconds(2), new TimeTest.Buncher.Batch(Arrays.asList(one, two)));

	}

}
