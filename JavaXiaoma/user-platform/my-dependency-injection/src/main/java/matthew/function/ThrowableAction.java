package matthew.function;

/**
 * @author Matthew
 * @projectName Validation
 * @time 2021/10/20 10:48/**
 *  * A function interface for action with {@link Throwable}
 *  *
 *  * @see Function
 *  * @see Throwable
 *
 */

@FunctionalInterface
public interface ThrowableAction {


    /**
     * Executes the action
     *
     * @throws Throwable if met with error
     */
    void execute() throws Throwable;


    /**
     * Executes {@link ThrowableAction}
     *
     * @param action {@link ThrowableAction}
     * @throws RuntimeException wrap {@link Exception} to {@link RuntimeException}
     */
    static void execute(ThrowableAction action) throws RuntimeException{
        try {
            action.execute();
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }
}

