package io.matt.simple.interaction;

import akka.actor.typed.ActorSystem;
import org.junit.Test;

/**
 * @author Matthew Hua
 * @Date 2023/5/16
 */
public class TestMain {

	@Test
	public void askWithStatusExample() {

		ActorSystem<InteractionPatternsAskWithStatusTest.Samples.Hal.Command> actorSystem =
				ActorSystem.create(InteractionPatternsAskWithStatusTest.Samples.Hal.create(), "SHY11");


	}
}
