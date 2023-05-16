package io.matt.simple.interaction;

import akka.actor.typed.ActorRef;
import akka.actor.typed.ActorSystem;
import akka.actor.typed.javadsl.AskPattern;
import akka.pattern.StatusReply;

import java.time.Duration;
import java.util.concurrent.CompletionStage;

/**
 * @author Matthew Hua
 * @Date 2023/5/16
 */
public class NotShown {

	public void askAndPrint(
			ActorSystem<Void> system, ActorRef<StandaloneAskSample.CookieFabric.CommandOne> cookieFabric) {
		CompletionStage<StandaloneAskSample.CookieFabric.Cookies> result =
				AskPattern.askWithStatus(
						cookieFabric,
						replyTo -> new StandaloneAskSample.CookieFabric.GiveMeCookies(3, replyTo),
						// asking someone requires a timeout and a scheduler, if the timeout hits without
						// response the ask is failed with a TimeoutException
						Duration.ofSeconds(3),
						system.scheduler());

		result.whenComplete(
				(reply, failure) -> {
					if (reply != null) System.out.println("Yay, " + reply.count + " cookies!");
					else if (failure instanceof StatusReply.ErrorMessage)
						System.out.println("No cookies for me. " + failure.getMessage());
					else System.out.println("Boo! didn't get cookies in time. " + failure);
				});
	}
}
