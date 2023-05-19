package io.matt.simple.interaction;

import akka.Done;
import akka.actor.typed.ActorRef;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import lombok.AccessLevel;
import lombok.AllArgsConstructor;

import java.util.concurrent.CompletionStage;

/**
 * @author Matthew Hua
 * @Date 2023/5/18
 */
public interface PipeToSelfSample {

	interface CustomerDataAccess {
		CompletionStage<Done> update(Customer customer);
	}


	@AllArgsConstructor(access = AccessLevel.PUBLIC)
	class Customer {
	 final String id;
	 final long version;
	 final String name;
	 final String address;
	}

	class CustomerRepository extends AbstractBehavior<CustomerRepository.Command> {

		private static final int MAX_OPERATIONS_IN_PROGRESS = 10;



		interface Command {}

		@AllArgsConstructor(access = AccessLevel.PUBLIC)
		public static class Update implements Command {
			final Customer customer;
			final ActorRef<OperationResult> replyTo;
		}


		interface OperationResult {}

		@AllArgsConstructor
		public static class UpdateSuccess implements OperationResult {
			public final String id;
		}

		@AllArgsConstructor(access = AccessLevel.PUBLIC)
		public static class UpdateFailure implements OperationResult {
			final String id;
			final String reason;
		}
		@AllArgsConstructor(access = AccessLevel.PUBLIC)
		private static class WrappedUpdateResult implements Command {
			final OperationResult result;
			final ActorRef<OperationResult> replyTo;
		}


		public static Behavior<Command> create(CustomerDataAccess dataAccess) {
			return Behaviors.setup(context -> new CustomerRepository(context, dataAccess));
		}

		private final CustomerDataAccess dataAccess;
		private int operationInProgress = 0;

		public CustomerRepository(ActorContext<Command> context, CustomerDataAccess dataAccess) {
			super(context);
			this.dataAccess = dataAccess;
		}

		@Override
		public Receive<Command> createReceive() {
			return newReceiveBuilder()
					.onMessage(Update.class, this::onUpdate)
					.onMessage(WrappedUpdateResult.class, this::onUpdateResult)
					.build();
		}

		private Behavior<Command> onUpdate(Update command) {
			if (operationInProgress == MAX_OPERATIONS_IN_PROGRESS) {
				command.replyTo.tell(
						new UpdateFailure(
								command.customer.id,
								"Max " + MAX_OPERATIONS_IN_PROGRESS + "concurrent operations supported"));
			} else {
				// increase operationInProgress counter
				operationInProgress++;
				CompletionStage<Done> futureResult = dataAccess.update(command.customer);
				getContext()
						.pipeToSelf(
								futureResult,
								(ok, exc) -> {
									if (exc == null)
										return new WrappedUpdateResult(
												new UpdateSuccess(command.customer.id), command.replyTo);
									else
										return new WrappedUpdateResult(
												new UpdateFailure(command.customer.id, exc.getMessage()), command.replyTo);
								});
			}
			return this;
		}

		private Behavior<CustomerRepository.Command> onUpdateResult(CustomerRepository.WrappedUpdateResult wrapped) {
			// decrease operationsInProgress counter
			operationInProgress--;
			// send result to original requestor
			wrapped.replyTo.tell(wrapped.result);
			return this;
		}

	}



}
