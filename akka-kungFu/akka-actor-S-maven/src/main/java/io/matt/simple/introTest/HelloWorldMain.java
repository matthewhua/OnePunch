package io.matt.simple.introTest;

import akka.actor.typed.ActorRef;
import akka.actor.typed.ActorSystem;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AllArgsConstructor;

public class HelloWorldMain extends AbstractBehavior<HelloWorldMain.sayHello> {

	@AllArgsConstructor
	public static class sayHello {
		public final String name;
	}

	public static Behavior<sayHello> create() {
		return Behaviors.setup(HelloWorldMain::new);
	}

	private final ActorRef<HelloWorld.Greet> greeter;

	public HelloWorldMain(ActorContext<sayHello> context) {
		super(context);
		greeter = context.spawn(HelloWorld.create(), "greeter");
	}


	@Override
	public Receive<sayHello> createReceive() {
		return newReceiveBuilder().onMessage(sayHello.class, this::onStart).build();
	}

	private Behavior<sayHello> onStart(sayHello command) {
		ActorRef<HelloWorld.Greeted> replyTo = getContext().spawn(HelloWorldBot.create(3), command.name);
		greeter.tell(new HelloWorld.Greet(command.name, replyTo));
		return this;
	}

	static class test {
		public static void main(String[] args) throws InterruptedException {
			ActorSystem<sayHello> system = ActorSystem.create(HelloWorldMain.create(), "hello");
			system.tell(new HelloWorldMain.sayHello("World"));
			system.tell(new HelloWorldMain.sayHello("Akka"));

			Thread.sleep(3000);
			system.terminate();
		}
	}
}


