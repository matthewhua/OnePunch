package io.matt.simple.interaction;

import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AccessLevel;
import lombok.AllArgsConstructor;

import java.time.Duration;
import java.util.Optional;

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

	public interface KeyCabinet {

		public class Keys {
		}

		public class Wallet {
		}

		@AllArgsConstructor
		public static class GetKeys {
			public final String whoseKeys;
			public final ActorRef<Keys> replyTo;
		}

		public static Behavior<GetKeys> create() {
			return Behaviors.receiveMessage(KeyCabinet::onGetKeys);
		}

		private static Behavior<GetKeys> onGetKeys(GetKeys message) {
			message.replyTo.tell(new Keys());
			return Behaviors.same();
		}

		class Drawer {
			@AllArgsConstructor
			public static class GetWallet {
				public final String whoseWallet;
				public final ActorRef<Wallet> replyTo;
			}

			public static Behavior<GetWallet> create() {
				return Behaviors.receiveMessage(Drawer::onGetWallet);
			}

			private static Behavior<GetWallet> onGetWallet(GetWallet message) {
				message.replyTo.tell(new Wallet());
				return Behaviors.same();
			}
		}

		class Home {
			public interface Command {};

			@AllArgsConstructor(access = AccessLevel.PUBLIC)
			public static class LeaveHome implements Command {
				final String who;
				final ActorRef<ReadyToLeaveHome> responseTo;
			}

			@AllArgsConstructor(access = AccessLevel.PUBLIC)
			public static class ReadyToLeaveHome {
				final String who;
				final Keys keys;
				final Wallet wallet;
			}

			private final ActorContext<Command> context;

			private final ActorRef<KeyCabinet.GetKeys> keysCabinet;
			private final ActorRef<Drawer.GetWallet> drawer;

			public Home(ActorContext<Command> context) {
				this.context = context;
				this.keysCabinet = context.spawn(KeyCabinet.create(), "key-cabinet");
				this.drawer = context.spawn(Drawer.create(), "drawer");
			}

			private Behavior<Command> behavior() {
				return Behaviors.receive(Command.class)
						.onMessage(LeaveHome.class, this::onLeaveHome)
						.build();
			}

			private Behavior<Command> onLeaveHome(LeaveHome message) {
				context.spawn(
						PrepareToLeaveHome.create(message.who, message.responseTo, keysCabinet, drawer),
						"leaving" + message.who);
				return Behaviors.same();
			}

		}

		class PrepareToLeaveHome extends AbstractBehavior<Object> {
			static Behavior<Object> create(
					String whoIsLeaving,
					ActorRef<Home.ReadyToLeaveHome> replyTo,
					ActorRef<KeyCabinet.GetKeys> keyCabinet,
					ActorRef<Drawer.GetWallet> drawer) {
				return Behaviors.setup(
						context -> new PrepareToLeaveHome(context, whoIsLeaving, replyTo, keyCabinet, drawer));
			}

			private final String whoIsLeaving;
			private final ActorRef<Home.ReadyToLeaveHome> replyTo;
			private final ActorRef<KeyCabinet.GetKeys> keyCabinet;
			private final ActorRef<Drawer.GetWallet> drawer;
			private Optional<Wallet> wallet = Optional.empty();
			private Optional<Keys> keys = Optional.empty();

			private PrepareToLeaveHome(
					ActorContext<Object> context,
					String whoIsLeaving,
					ActorRef<Home.ReadyToLeaveHome> replyTo,
					ActorRef<KeyCabinet.GetKeys> keyCabinet,
					ActorRef<Drawer.GetWallet> drawer) {
				super(context);
				this.whoIsLeaving = whoIsLeaving;
				this.replyTo = replyTo;
				this.keyCabinet = keyCabinet;
				this.drawer = drawer;
			}

			@Override
			public Receive<Object> createReceive() {
				return newReceiveBuilder()
						.onMessage(Wallet.class, this::onWallet)
						.onMessage(Keys.class, this::onKeys)
						.build();
			}

			private Behavior<Object> onWallet(Wallet wallet) {
				this.wallet = Optional.of(wallet);
				return completeOrContinue();
			}

			private Behavior<Object> onKeys(Keys keys) {
				this.keys = Optional.of(keys);
				return completeOrContinue();
			}

			private Behavior<Object> completeOrContinue() {
				if (wallet.isPresent() && keys.isPresent()) {
					replyTo.tell(new Home.ReadyToLeaveHome(whoIsLeaving, keys.get(), wallet.get()));
					return Behaviors.stopped();
				} else {
					return this;
				}
			}

		}
	}

}
