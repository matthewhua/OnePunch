package io.matt.simple.interaction;

import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import akka.pattern.StatusReply;
import lombok.AllArgsConstructor;

/**
 * @author Matthew Hua
 * @Date 2023/5/16
 */
public interface StandaloneAskSample {

	class CookieFabric extends AbstractBehavior<CookieFabric.CommandOne> {

		interface CommandOne{}

		@AllArgsConstructor
		public static class GiveMeCookies implements CookieFabric.CommandOne {
			public final int count;
			public final ActorRef<StatusReply<CookieFabric.Cookies>> replyTo;
		}

		@AllArgsConstructor
		public static class Cookies {
			public final int count;
		}

		public static Behavior<CookieFabric.CommandOne> createOne() {
			return Behaviors.setup(CookieFabric::new);
		}

		@Override
		public Receive<CommandOne> createReceive() {
			return newReceiveBuilder()
					.onMessage(CookieFabric.GiveMeCookies.class, this::onGiveMeCookies)
					.build();
		}

		private Behavior<CookieFabric.CommandOne> onGiveMeCookies(CookieFabric.GiveMeCookies request) {
			if (request.count >= 5) request.replyTo.tell(StatusReply.error("Too many cookies."));
			else request.replyTo.tell(StatusReply.success(new CookieFabric.Cookies(request.count)));

			return this;
		}

		public CookieFabric(ActorContext<CommandOne> context) {
			super(context);
		}

	}
}
