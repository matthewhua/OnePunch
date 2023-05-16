package io.matt.simple.interaction;

import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AllArgsConstructor;

import java.time.Duration;

/**
 * @author Matthew Hua
 * @Date 2023/5/16
 */
public class Dave extends AbstractBehavior<Dave.Command> {

	public interface Command {
	}

	@AllArgsConstructor
	private static final class AdaptedResponse implements Dave.Command {
		public final String message;
	}

	public static Behavior<Dave.Command> create(ActorRef<InteractionPatternsAskWithStatusTest.Samples.Hal.Command> hal) {
		return Behaviors.setup(context -> new Dave(context, hal));
	}

	public Dave(ActorContext<Dave.Command> context, ActorRef<InteractionPatternsAskWithStatusTest.Samples.Hal.Command> hal) {
		super(context);

		Duration timeOut = Duration.ofSeconds(3);

		context.askWithStatus(
				String.class,
				hal,
				timeOut,
				// construct the outgoing message
				InteractionPatternsAskWithStatusTest.Samples.Hal.OpenThePodBayDoorPlease::new,
				// adapt the response (or failure to respond)
				(response, throwable) -> {
					if (response != null) {
						// a ResponseWithStatus.success(m) is unwrapped and passed as response
						return new Dave.AdaptedResponse(response);
					} else {
						// a ResponseWithStatus.error will end up as a StatusReply.ErrorMessage()
						// exception here
						return new Dave.AdaptedResponse("Request failed:" + throwable.getMessage());
					}
				}
		);
	}

	@Override
	public Receive<Command> createReceive() {
		return newReceiveBuilder()
				// the adapted message ends up being processed like any other
				// message sent to the actor
				.onMessage(Dave.AdaptedResponse.class, this::onAdaptedResponse)
				.build();
	}

	private Behavior<Dave.Command> onAdaptedResponse(Dave.AdaptedResponse response) {
		getContext().getLog().info("Got response from HAL : {}", response.message);
		return this;
	}

}
