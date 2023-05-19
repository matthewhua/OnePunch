package io.matt.simple.interaction;

import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.Behaviors;
import lombok.AllArgsConstructor;

/**
 * @author Matthew Hua
 * @Date 2023/5/19
 */
public interface FireAndForget {

	// #fire-and-forget-definition

	class Printer {

		@AllArgsConstructor
		public static class PrintMe {
			public final String message;
		}


		public static Behavior<PrintMe> create() {
			return Behaviors.setup(
					context ->
							Behaviors.receive(PrintMe.class)
									.onMessage(
											PrintMe.class,
											printMe -> {
												context.getLog().info(printMe.message);
												return Behaviors.same();
											}).build());
		}
	}


}
