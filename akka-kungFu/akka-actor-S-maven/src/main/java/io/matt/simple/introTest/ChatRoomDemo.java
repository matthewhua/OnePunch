package io.matt.simple.introTest;

import akka.actor.typed.ActorRef;
import akka.actor.typed.ActorSystem;
import akka.actor.typed.Behavior;
import akka.actor.typed.Terminated;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import lombok.AllArgsConstructor;
import lombok.Data;

import java.io.UnsupportedEncodingException;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/**
 * @author Matthew Hua
 * @data 2023/5/12
 */
public interface ChatRoomDemo {

	public class ChatRoom {

		// #chatroom-protocol
		static interface RoomCommand {
		}

		@Data
		public static final class GetSession implements RoomCommand {
			public final String screenName;
			public final ActorRef<SessionEvent> replyTo;
		}
		// #chatroom-protocol

		@Data
		private static final class PublishSessionMessage implements RoomCommand {
			public final String screenName;
			public final String message;
		}

		// #chatroom-protocol
		interface SessionEvent {
		}

		@Data
		public static final class SessionGranted implements SessionEvent {
			public final ActorRef<PostMessage> handle;
		}

		@Data
		public static final class SessionDenied implements SessionEvent {
			public final String reason;
		}

		@Data
		public static final class MessagePosted implements SessionEvent {
			public final String screenName;
			public final String message;
		}

		interface SessionCommand {
		}

		@Data
		public static final class PostMessage implements SessionCommand {
			public final String message;
		}

		@AllArgsConstructor
		private static final class NotifyClient implements SessionCommand {
			final MessagePosted messagePosted;
		}

		// #chatroom-protocol

		// #chatroom-behavior
		public static Behavior<RoomCommand> create() {
			return Behaviors.setup(ctx -> {
				return new ChatRoom(ctx).chatRoom(new ArrayList<ActorRef<SessionCommand>>());
			});
		}

		private final ActorContext<RoomCommand> context;

		private ChatRoom(ActorContext<RoomCommand> context) {
			this.context = context;
		}

		private Behavior<RoomCommand> chatRoom(List<ActorRef<SessionCommand>> sessions) {
			return Behaviors.receive(RoomCommand.class)
					.onMessage(GetSession.class, getSession -> onGetSession(sessions, getSession))
					.onMessage(PublishSessionMessage.class, pub -> onPublishSessionMessage(sessions, pub))
					.build();
		}

		private Behavior<RoomCommand> onGetSession(List<ActorRef<SessionCommand>> sessions, GetSession getSession) throws UnsupportedEncodingException {
			ActorRef<SessionEvent> client = getSession.replyTo;
			ActorRef<SessionCommand> spawn = context.spawn(
					Session.create(context.getSelf(), getSession.screenName, client),
					URLEncoder.encode(getSession.screenName, StandardCharsets.UTF_8.name()));

			// narrow to only expose PostMessage
			client.tell(new SessionGranted(spawn.narrow()));
			List<ActorRef<SessionCommand>> newSeesions = new ArrayList<>(sessions);
			newSeesions.add(spawn);
			return chatRoom(newSeesions);
		}

		private Behavior<RoomCommand> onPublishSessionMessage(List<ActorRef<SessionCommand>> sessions, PublishSessionMessage pub) throws UnsupportedEncodingException {
			NotifyClient notifyClient = new NotifyClient(new MessagePosted(pub.screenName, pub.message));
			sessions.forEach(s -> s.tell(notifyClient));
			return Behaviors.same();
		}

		static class Session {
			static Behavior<ChatRoom.SessionCommand> create(
					ActorRef<RoomCommand> room, String screenName, ActorRef<SessionEvent> client) {
				return Behaviors.receive(ChatRoom.SessionCommand.class)
						.onMessage(PostMessage.class, post -> onPostMessage(room, screenName, post))
						.onMessage(NotifyClient.class, notification -> onNotifyClient(client, notification))
						.build();
			}

			private static Behavior<SessionCommand> onPostMessage(
					ActorRef<RoomCommand> room, String screenName, PostMessage post) {
				// from client, publish to others via the room
				room.tell(new PublishSessionMessage(screenName, post.message));
				return Behaviors.same();
			}

			private static Behavior<SessionCommand> onNotifyClient(
					ActorRef<SessionEvent> client, NotifyClient notifyClient) {
				// published from the room
				client.tell(notifyClient.messagePosted);
				return Behaviors.same();
			}
		}

	}


	// #chatroom-gabbler
	public class Gabbler {
		public static Behavior<ChatRoom.SessionEvent> create() {
			return Behaviors.setup(ctx -> new Gabbler(ctx).behavior());
		}

		private final ActorContext<ChatRoom.SessionEvent> context;

		public Gabbler(ActorContext<ChatRoom.SessionEvent> context) {
			this.context = context;
		}

		private Behavior<ChatRoom.SessionEvent> behavior() {
			return Behaviors.receive(ChatRoom.SessionEvent.class)
					.onMessage(ChatRoom.SessionDenied.class, this::onSessionDenied)
					.onMessage(ChatRoom.SessionGranted.class, this::onSessionGranted)
					.onMessage(ChatRoom.MessagePosted.class, this::onMessagePosted)
					.build();
		}

		private Behavior<ChatRoom.SessionEvent> onSessionDenied(ChatRoom.SessionDenied message) {
			context.getLog().info("cannot start chat room session: {}", message.reason);
			return Behaviors.stopped();
		}

		private Behavior<ChatRoom.SessionEvent> onSessionGranted(ChatRoom.SessionGranted message) {
			message.handle.tell(new ChatRoom.PostMessage("Hello World!"));
			return Behaviors.same();
		}

		private Behavior<ChatRoom.SessionEvent> onMessagePosted(ChatRoom.MessagePosted message) {
			context
					.getLog()
					.info("message has been posted by '{}': {}", message.screenName, message.message);
			return Behaviors.stopped();
		}
	}


	public class Main {

		public static Behavior<Void> create() {
			return Behaviors.setup(context -> {
				ActorRef<ChatRoom.RoomCommand> chatRoom = context.spawn(ChatRoom.create(), "chatRoom");
				ActorRef<ChatRoom.SessionEvent> gabbler = context.spawn(Gabbler.create(), "gabbler");
				context.watch(gabbler);
				chatRoom.tell(new ChatRoom.GetSession("ol' Gabbler", gabbler));

				return Behaviors.receive(Void.class)
						.onSignal(Terminated.class, sig -> Behaviors.stopped())
						.build();
			});
		}

		public static void main(String[] args) {
			ActorSystem.create(Main.create(), "ChatRoomDemo");
		}
	}
}
